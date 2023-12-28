#[cfg(test)]
mod tests {
    use super::*;
    use compiler::pipeline::Pipeline;
    use mktemp::Temp;
    use number::GoldilocksField;
    use riscv::{
        compile_rust,
        continuations::{rust_continuations, rust_continuations_dry_run},
        CoProcessors,
    };
    use std::path::PathBuf;
    #[test]
    #[ignore]
    fn compile_rust_riscv() {
        env_logger::try_init().unwrap_or_default();
        let temp_dir = Temp::new_dir().unwrap();
        log::info!("Write to {:?}", temp_dir);
        let case = "tests/evm";
        let coprocessors = CoProcessors::base().with_poseidon();
        let powdr_asm = compile_rust(case, &temp_dir, true, &coprocessors, true).unwrap();

        let pipeline = Pipeline::default().from_asm_string(powdr_asm.1, Some(PathBuf::from(case)));
        rust_continuations_dry_run::<GoldilocksField>(
            pipeline,
            [11, 97, 2, 154, 96, 0, 82, 96, 32, 96, 0, 243]
                .map(|i| GoldilocksField::from(i))
                .into(),
        );
    }
}
