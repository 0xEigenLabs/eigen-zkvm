use number::GoldilocksField;
use riscv::compile_rust;
use std::path::Path;

fn main() {
    let file = "src/regression.rs";

    let _result = compile_rust::<GoldilocksField>(
        file,
        Vec::new(),
        &Path::new("test_regression/"),
        true,
        None,
    );
}
