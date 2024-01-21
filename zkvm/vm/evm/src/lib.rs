#![no_std]
use revm::{
    db::CacheState,
    interpreter::CreateScheme,
    primitives::{
        Address,
        calc_excess_blob_gas, keccak256, Env, AccountInfo, Bytecode, TransactTo, U256, SpecId
    },
};
//use runtime::{print, get_prover_input, coprocessors::{get_data, get_data_len}};
use powdr_riscv_rt::{print, coprocessors::get_data_serde};

use models::*;

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::string::ToString;

use k256::ecdsa::SigningKey;

/// Recover the address from a private key (SigningKey).
pub fn recover_address(private_key: &[u8]) -> Option<Address> {
    let key = SigningKey::from_slice(private_key).ok()?;
    let public_key = key.verifying_key().to_encoded_point(false);
    Some(Address::from_raw_public_key(&public_key.as_bytes()[1..]))
}

#[no_mangle]
fn main() {
    let suite_json: String = get_data_serde(666);
    print!("suite_json: {suite_json}\n");
    let suite = read_suite(&suite_json);

    /*
    let addr = address!("a94f5374fce5edbc8e2a8697c15331677e6ebf0b");
    */
    assert!(execute_test(&suite).is_ok());
}

fn read_suite(s: &String) -> TestUnit {
    let suite: TestUnit = serde_json::from_str(s).map_err(|e| e).unwrap();
    suite
}

fn execute_test(unit: &TestUnit) -> Result<(), String> {
    // Create database and insert cache
    let mut cache_state = CacheState::new(false);
    for (address, info) in &unit.pre {
        let acc_info = AccountInfo {
            balance: info.balance,
            code_hash: keccak256(&info.code),
            code: Some(Bytecode::new_raw(info.code.clone())),
            nonce: info.nonce,
        };
        cache_state.insert_account_with_storage(*address, acc_info, info.storage.clone());
    }

    let mut env = Env::default();
    env.cfg.chain_id = match unit.chain_id {
        Some(chain_id) => chain_id,
        _ => 1, // mainnet by default
    };
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
    env.tx.caller = match unit.transaction.sender {
            Some(address) => address,
            _ => recover_address(unit.transaction.secret_key.as_slice())
                .ok_or_else(|| String::new())?,
        };
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

        //env.cfg.spec_id = spec_name.to_spec_id();

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
            let spec_id = spec_name.to_spec_id();

             let mut cache = cache_state.clone();
                cache.set_state_clear_flag(SpecId::enabled(
                    spec_id,
                    revm::primitives::SpecId::SPURIOUS_DRAGON,
                ));
            let mut state = revm::db::State::builder()
                .with_cached_prestate(cache)
                .with_bundle_update()
                .build();

            let mut evm = revm::Evm::builder()
                    .with_db(&mut state)
                    .modify_env(|e| *e = env.clone())
                    .spec_id(spec_id)
                    .build();

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
                    (Some(_), Err(_e)) => {
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
