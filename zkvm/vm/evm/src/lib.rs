#![no_std]

use revm::{
    db::CacheState,
    interpreter::CreateScheme,
    primitives::{
        address, b256, calc_excess_blob_gas, keccak256, Env, HashMap, SpecId, ruint::Uint, AccountInfo, Address, Bytecode, Bytes, TransactTo, B256, U256,
    },
    EVM,
};
//use runtime::{print, get_prover_input, coprocessors::{get_data, get_data_len}};
use powdr_riscv_rt::{print, coprocessors::{get_data, get_data_len}};

use models::*;

extern crate alloc;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::string::ToString;

#[no_mangle]
fn main() {
    ethereum_tests_simple();
}

fn ethereum_tests_simple() {
    let suite_len = get_data_len(666);
    let mut suite_json = vec![0; suite_len];
    get_data(666, &mut suite_json);

    let suite_json: Vec<u8> = suite_json.into_iter().map(|x| x as u8).collect();

    let suite_json_str = String::from_utf8(suite_json).unwrap();
    let suite = read_suite(&suite_json_str);

    assert!(execute_test(&suite).is_ok());
}

fn read_suite(s: &String) -> TestUnit {
    let suite: TestUnit = serde_json::from_str(s).map_err(|e| e).unwrap();
    suite
}

fn execute_test(unit: &TestUnit) -> Result<(), String> {
    let map_caller_keys: HashMap<_, _> = [
        (
            b256!("45a915e4d060149eb4365960e6a7a45f334393093061116b197e3240065ff2d8"),
            address!("a94f5374fce5edbc8e2a8697c15331677e6ebf0b"),
        ),
        (
            b256!("c85ef7d79691fe79573b1a7064c19c1a9819ebdbd1faaab1a8ec92344438aaf4"),
            address!("cd2a3d9f938e13cd947ec05abc7fe734df8dd826"),
        ),
        (
            b256!("044852b2a670ade5407e78fb2863c51de9fcb96542a07186fe3aeda6bb8a116d"),
            address!("82a978b3f5962a5b0957d9ee9eef472ee55b42f1"),
        ),
        (
            b256!("6a7eeac5f12b409d42028f66b0b2132535ee158cfda439e3bfdd4558e8f4bf6c"),
            address!("c9c5a15a403e41498b6f69f6f89dd9f5892d21f7"),
        ),
        (
            b256!("a95defe70ebea7804f9c3be42d20d24375e2a92b9d9666b832069c5f3cd423dd"),
            address!("3fb1cd2cd96c6d5c0b5eb3322d807b34482481d4"),
        ),
        (
            b256!("fe13266ff57000135fb9aa854bbfe455d8da85b21f626307bf3263a0c2a8e7fe"),
            address!("dcc5ba93a1ed7e045690d722f2bf460a51c61415"),
        ),
    ]
    .into();

        // Create database and insert cache
        let mut cache_state = CacheState::new(false);
        for (address, info) in &unit.pre {
            let acc_info = revm::primitives::AccountInfo {
                balance: info.balance,
                code_hash: keccak256(&info.code),
                code: Some(Bytecode::new_raw(info.code.clone())),
                nonce: info.nonce,
            };
            cache_state.insert_account_with_storage(*address, acc_info, info.storage.clone());
        }

        let mut env = Env::default();
        // for mainnet
        env.cfg.chain_id = 1;
        // env.cfg.spec_id is set down the road

        // block env
        env.block.number = unit.env.current_number;
        env.block.coinbase = unit.env.current_coinbase;
        env.block.timestamp = unit.env.current_timestamp;
        env.block.gas_limit = unit.env.current_gas_limit;
        env.block.basefee = unit.env.current_base_fee.unwrap_or_default();
        env.block.difficulty = unit.env.current_difficulty;
        // after the Merge prevrandao replaces mix_hash field in block and replaced difficulty opcode in EVM.
        env.block.prevrandao = Some(unit.env.current_difficulty.to_be_bytes().into());
        // EIP-4844
        if let (Some(parent_blob_gas_used), Some(parent_excess_blob_gas)) = (
            unit.env.parent_blob_gas_used,
            unit.env.parent_excess_blob_gas,
        ) {
            env.block
                .set_blob_excess_gas_and_price(calc_excess_blob_gas(
                    parent_blob_gas_used.to(),
                    parent_excess_blob_gas.to(),
                ));
        }

        // tx env
        let pk = unit.transaction.secret_key;
        env.tx.caller = map_caller_keys.get(&pk).copied().ok_or_else(|| String::new())?;
        env.tx.gas_price = unit
            .transaction
            .gas_price
            .or(unit.transaction.max_fee_per_gas)
            .unwrap_or_default();
        env.tx.gas_priority_fee = unit.transaction.max_priority_fee_per_gas;
        // EIP-4844
        env.tx.blob_hashes = unit.transaction.blob_versioned_hashes.clone();
        env.tx.max_fee_per_blob_gas = unit.transaction.max_fee_per_blob_gas;

        // post and execution
        for (spec_name, tests) in &unit.post {
            if matches!(
                spec_name,
                SpecName::ByzantiumToConstantinopleAt5
                    | SpecName::Constantinople
                    | SpecName::Unknown
            ) {
                continue;
            }

            env.cfg.spec_id = spec_name.to_spec_id();

            for test in tests {
                env.tx.gas_limit = unit.transaction.gas_limit[test.indexes.gas].saturating_to();

                env.tx.data = unit
                    .transaction
                    .data
                    .get(test.indexes.data)
                    .unwrap()
                    .clone();
                env.tx.value = unit.transaction.value[test.indexes.value];

                env.tx.access_list = unit
                    .transaction
                    .access_lists
                    .get(test.indexes.data)
                    .and_then(Option::as_deref)
                    .unwrap_or_default()
                    .iter()
                    .map(|item| {
                        (
                            item.address,
                            item.storage_keys
                                .iter()
                                .map(|key| U256::from_be_bytes(key.0))
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect();

                let to = match unit.transaction.to {
                    Some(add) => TransactTo::Call(add),
                    None => TransactTo::Create(CreateScheme::Create),
                };
                env.tx.transact_to = to;

                let mut cache = cache_state.clone();
                cache.set_state_clear_flag(SpecId::enabled(
                    env.cfg.spec_id,
                    revm::primitives::SpecId::SPURIOUS_DRAGON,
                ));
                let mut state = revm::db::State::builder()
                    .with_cached_prestate(cache)
                    .with_bundle_update()
                    .build();
                let mut evm = revm::new();
                evm.database(&mut state);
                evm.env = env.clone();

                // do the deed
                let exec_result = evm.transact_commit();

                // validate results
                // this is in a closure so we can have a common printing routine for errors
                let check = || {
                    // if we expect exception revm should return error from execution.
                    // So we do not check logs and state root.
                    //
                    // Note that some tests that have exception and run tests from before state clear
                    // would touch the caller account and make it appear in state root calculation.
                    // This is not something that we would expect as invalid tx should not touch state.
                    // but as this is a cleanup of invalid tx it is not properly defined and in the end
                    // it does not matter.
                    // Test where this happens: `tests/GeneralStateTests/stTransactionTest/NoSrcAccountCreate.json`
                    // and you can check that we have only two "hash" values for before and after state clear.
                    match (&test.expect_exception, &exec_result) {
                        // do nothing
                        (None, Ok(_)) => (),
                        // return okay, exception is expected.
                        (Some(_), Err(e)) => {
                            //print!("ERROR: {e}");
                            return Ok(());
                        }
                        _ => {
                            let s = exec_result.clone().err().map(|e| e.to_string()).unwrap();
                            print!("UNEXPECTED ERROR: {s}");
                            return Err(s);
                        }
                    }
                    Ok(())
                };

                // dump state and traces if test failed
                let Err(e) = check() else { continue };

                return Err(e);
            }
        }
    Ok(())
}
