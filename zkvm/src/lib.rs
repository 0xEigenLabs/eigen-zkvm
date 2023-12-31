#![allow(unused_macros)]
#![allow(dead_code)]
use std::io::BufWriter;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

macro_rules! local_fill {
    ($left:expr, $right:expr, $fun:expr) => {
        if let Some(right) = $right {
            $left = $fun(right.0)
        }
    };
    ($left:expr, $right:expr) => {
        if let Some(right) = $right {
            $left = Address::from(right.as_fixed_bytes())
        }
    };
}

struct FlushWriter {
    writer: Arc<Mutex<BufWriter<std::fs::File>>>,
}

impl FlushWriter {
    fn new(writer: Arc<Mutex<BufWriter<std::fs::File>>>) -> Self {
        Self { writer }
    }
}

impl Write for FlushWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.lock().unwrap().flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use backend::BackendType;
    use compiler::pipeline::{Pipeline, Stage};
    use ethers_providers::Middleware;
    use ethers_providers::{Http, Provider};
    use indicatif::ProgressBar;
    use mktemp::Temp;
    use number::GoldilocksField;
    use revm::db::{CacheDB, EmptyDB};
    use revm::inspectors::TracerEip3155;
    use revm::primitives::{Address, Env, TransactTo, U256};
    use revm::EVM;
    use riscv::{
        compile_rust,
        continuations::{rust_continuations, rust_continuations_dry_run},
        CoProcessors,
    };
    use std::fs::OpenOptions;
    use std::path::PathBuf;

    static BYTECODE: &str = "61029a60005260206000f3";

    #[test]
    fn test_revm_prove_single_contract() {
        env_logger::try_init().unwrap_or_default();

        type F = GoldilocksField;
        let temp_dir = Temp::new_dir().unwrap();
        log::info!("Write to {:?}", temp_dir);
        let case = "vm/evm";
        let coprocessors = CoProcessors::base().with_poseidon();
        // Compile REVM to powdr asm
        let powdr_asm = compile_rust(case, &temp_dir, true, &coprocessors, true).unwrap();

        let bytes = hex::decode(BYTECODE).unwrap();

        let length: GoldilocksField = (bytes.len() as u64).into();
        let mut bytecode: Vec<GoldilocksField> = vec![length];
        bytecode.extend(bytes.into_iter().map(|x| GoldilocksField::from(x as u64)));

        // Load the powdr asm
        let pipeline_factory = || {
            Pipeline::default()
                .from_asm_string(powdr_asm.1.clone(), Some(PathBuf::from(case)))
                .with_prover_inputs(bytecode.clone())
        };

        // Execute the evm and generate inputs for segment
        let bootloader_inputs =
            rust_continuations_dry_run::<GoldilocksField>(pipeline_factory(), bytecode.clone());

        // Build the wtns and proof
        let prove_with = Some(BackendType::EStark);
        let generate_witness_and_prove_maybe =
            |mut pipeline: Pipeline<F>| -> Result<(), Vec<String>> {
                pipeline.advance_to(Stage::GeneratedWitness).unwrap();
                prove_with.map(|backend| pipeline.with_backend(backend).proof().unwrap());
                Ok(())
            };

        rust_continuations(
            pipeline_factory,
            generate_witness_and_prove_maybe,
            bootloader_inputs,
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_revm_prove_full_block() {
        // Create ethers client and wrap it in Arc<M>
        let client = Provider::<Http>::try_from(
            "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27",
        )
        .unwrap();
        let client = Arc::new(client);

        // Params
        let chain_id: u64 = 1;
        let block_number = 0x5BAD55;

        // Fetch the transaction-rich block
        let block = match client.get_block_with_txs(block_number).await {
            Ok(Some(block)) => block,
            Ok(None) => panic!("Block not found"),
            Err(error) => panic!("Error: {:?}", error),
        };
        println!("Fetched block number: {}", block.number.unwrap().0[0]);
        //let previous_block_number = block_number - 1;

        // Use the previous block state as the db with caching
        //let _prev_id: BlockId = previous_block_number.into();
        // SAFETY: This cannot fail since this is in the top-level tokio runtime
        //let state_db = EthersDB::new(Arc::clone(&client), Some(prev_id)).expect("panic");
        let cache_db = CacheDB::new(EmptyDB::default());
        let mut evm: EVM<CacheDB<EmptyDB>> = EVM::new();
        evm.database(cache_db);

        let mut env = Env::default();
        if let Some(number) = block.number {
            let nn = number.0[0];
            env.block.number = U256::from(nn);
        }
        local_fill!(env.block.coinbase, block.author);
        local_fill!(env.block.timestamp, Some(block.timestamp), U256::from_limbs);
        local_fill!(
            env.block.difficulty,
            Some(block.difficulty),
            U256::from_limbs
        );
        local_fill!(env.block.gas_limit, Some(block.gas_limit), U256::from_limbs);
        if let Some(base_fee) = block.base_fee_per_gas {
            local_fill!(env.block.basefee, Some(base_fee), U256::from_limbs);
        }

        let txs = block.transactions.len();
        println!("Found {txs} transactions.");

        let console_bar = Arc::new(ProgressBar::new(txs as u64));
        let elapsed = std::time::Duration::ZERO;

        // Create the traces directory if it doesn't exist
        std::fs::create_dir_all("traces").expect("Failed to create traces directory");

        // Fill in CfgEnv
        env.cfg.chain_id = chain_id;
        for tx in block.transactions {
            env.tx.caller = Address::from(tx.from.as_fixed_bytes());
            env.tx.gas_limit = tx.gas.as_u64();
            local_fill!(env.tx.gas_price, tx.gas_price, U256::from_limbs);
            local_fill!(env.tx.value, Some(tx.value), U256::from_limbs);
            env.tx.data = tx.input.0.into();
            let mut gas_priority_fee = U256::ZERO;
            local_fill!(
                gas_priority_fee,
                tx.max_priority_fee_per_gas,
                U256::from_limbs
            );
            env.tx.gas_priority_fee = Some(gas_priority_fee);
            env.tx.chain_id = Some(chain_id);
            env.tx.nonce = Some(tx.nonce.as_u64());
            if let Some(access_list) = tx.access_list {
                env.tx.access_list = access_list
                    .0
                    .into_iter()
                    .map(|item| {
                        let new_keys: Vec<U256> = item
                            .storage_keys
                            .into_iter()
                            .map(|h256| U256::from_le_bytes(h256.0))
                            .collect();
                        (Address::from(item.address.as_fixed_bytes()), new_keys)
                    })
                    .collect();
            } else {
                env.tx.access_list = Default::default();
            }

            env.tx.transact_to = match tx.to {
                Some(to_address) => TransactTo::Call(Address::from(to_address.as_fixed_bytes())),
                None => TransactTo::create(),
            };

            evm.env = env.clone();

            // Construct the file writer to write the trace to
            let tx_number = tx.transaction_index.unwrap().0[0];
            let file_name = format!("traces/{}.json", tx_number);
            let write = OpenOptions::new().write(true).create(true).open(file_name);
            let inner = Arc::new(Mutex::new(BufWriter::new(
                write.expect("Failed to open file"),
            )));
            let writer = FlushWriter::new(Arc::clone(&inner));

            // Inspect and commit the transaction to the EVM
            let inspector = TracerEip3155::new(Box::new(writer), true, true);
            if let Err(error) = evm.inspect_commit(inspector) {
                println!("Got error: {:?}", error);
            }

            // Flush the file writer
            inner.lock().unwrap().flush().expect("Failed to flush file");

            console_bar.inc(1);
        }

        console_bar.finish_with_message("Finished all transactions.");
        println!(
            "Finished execution. Total CPU time: {:.6}s",
            elapsed.as_secs_f64()
        );
    }
}
