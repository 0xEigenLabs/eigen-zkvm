// copied and modified by https://github.com/arkworks-rs/circom-compat/blob/master/src/witness/circom.rs
use crate::errors::Result;
use wasmer::{Function, Instance, Store, Value};

#[derive(Clone, Debug)]
pub struct Wasm(Instance);

// pub trait CircomBase {
//     fn init(&self, sanity_check: bool) -> Result<()>;
//     fn func(&self, name: &str) -> &Function;
//     fn get_ptr_witness_buffer(&self) -> Result<u32>;
//     fn get_ptr_witness(&self, w: u32) -> Result<u32>;
//     fn get_signal_offset32(
//         &self,
//         p_sig_offset: u32,
//         component: u32,
//         hash_msb: u32,
//         hash_lsb: u32,
//     ) -> Result<()>;
//     fn set_signal(&self, c_idx: u32, component: u32, signal: u32, p_val: u32) -> Result<()>;
//     fn get_u32(&self, name: &str) -> Result<u32>;
//     // Only exists natively in Circom2, hardcoded for Circom
//     fn get_version(&self) -> Result<u32>;
// }
//
// pub trait Circom {
//     fn get_field_num_len32(&self) -> Result<u32>;
//     fn get_raw_prime(&self) -> Result<()>;
//     fn read_shared_rw_memory(&self, i: u32) -> Result<u32>;
//     fn write_shared_rw_memory(&self, i: u32, v: u32) -> Result<()>;
//     fn set_input_signal(&self, hmsb: u32, hlsb: u32, pos: u32) -> Result<()>;
//     fn get_witness(&self, i: u32) -> Result<()>;
//     fn get_witness_size(&self) -> Result<u32>;
// }

// impl Circom for Wasm {
impl Wasm {
    pub(crate) fn get_field_num_len32(&self) -> Result<u32> {
        self.get_u32("getFieldNumLen32")
    }

    pub(crate) fn get_raw_prime(&self) -> Result<()> {
        let func = self.func("getRawPrime");
        let mut store = Store::default();
        func.call(&mut store, &[])?;
        Ok(())
    }

    pub(crate) fn read_shared_rw_memory(&self, i: u32) -> Result<u32> {
        let func = self.func("readSharedRWMemory");
        let mut store = Store::default();
        let result = func.call(&mut store, &[i.into()])?;
        Ok(result[0].unwrap_i32() as u32)
    }

    pub(crate) fn write_shared_rw_memory(&self, i: u32, v: u32) -> Result<()> {
        let func = self.func("writeSharedRWMemory");
        let mut store = Store::default();
        func.call(&mut store, &[i.into(), v.into()])?;
        Ok(())
    }

    pub(crate) fn set_input_signal(&self, hmsb: u32, hlsb: u32, pos: u32) -> Result<()> {
        let func = self.func("setInputSignal");
        let mut store = Store::default();
        func.call(&mut store, &[hmsb.into(), hlsb.into(), pos.into()])?;
        Ok(())
    }

    pub(crate) fn get_witness(&self, i: u32) -> Result<()> {
        let func = self.func("getWitness");
        let mut store = Store::default();
        func.call(&mut store, &[i.into()])?;
        Ok(())
    }

    pub(crate) fn get_witness_size(&self) -> Result<u32> {
        self.get_u32("getWitnessSize")
    }
    // }
    //
    // impl CircomBase for Wasm {
    pub(crate) fn init(&self, sanity_check: bool) -> Result<()> {
        let func = self.func("init");
        let mut store = Store::default();
        func.call(&mut store, &[Value::I32(sanity_check as i32)])?;
        Ok(())
    }

    pub(crate) fn get_ptr_witness_buffer(&self) -> Result<u32> {
        self.get_u32("getWitnessBuffer")
    }

    pub(crate) fn get_ptr_witness(&self, w: u32) -> Result<u32> {
        let func = self.func("getPWitness");
        let mut store = Store::default();
        let res = func.call(&mut store, &[w.into()])?;

        Ok(res[0].unwrap_i32() as u32)
    }

    pub(crate) fn get_signal_offset32(
        &self,
        p_sig_offset: u32,
        component: u32,
        hash_msb: u32,
        hash_lsb: u32,
    ) -> Result<()> {
        let func = self.func("getSignalOffset32");
        let mut store = Store::default();
        func.call(
            &mut store,
            &[
                p_sig_offset.into(),
                component.into(),
                hash_msb.into(),
                hash_lsb.into(),
            ],
        )?;

        Ok(())
    }

    pub(crate) fn set_signal(
        &self,
        c_idx: u32,
        component: u32,
        signal: u32,
        p_val: u32,
    ) -> Result<()> {
        let func = self.func("setSignal");
        let mut store = Store::default();
        func.call(
            &mut store,
            &[c_idx.into(), component.into(), signal.into(), p_val.into()],
        )?;

        Ok(())
    }

    // Default to version 1 if it isn't explicitly defined
    pub(crate) fn get_version(&self) -> Result<u32> {
        match self.0.exports.get_function("getVersion") {
            Ok(func) => {
                let mut store = Store::default();
                Ok(func.call(&mut store, &[])?[0].unwrap_i32() as u32)
            }
            Err(_) => Ok(1),
        }
    }

    pub(crate) fn get_u32(&self, name: &str) -> Result<u32> {
        let func = self.func(name);
        let mut store = Store::default();
        let result = func.call(&mut store, &[])?;
        Ok(result[0].unwrap_i32() as u32)
    }

    pub(crate) fn func(&self, name: &str) -> &Function {
        self.0
            .exports
            .get_function(name)
            .unwrap_or_else(|_| panic!("function {} not found", name))
    }
    // }
    //
    // impl Wasm {
    pub fn new(instance: Instance) -> Self {
        Self(instance)
    }
}
