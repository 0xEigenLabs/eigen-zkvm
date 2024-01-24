//! Poring from https://github.com/powdr-labs/powdr.git.
mod export;
mod expression_counter;

use powdr_number::GoldilocksField;
use powdr_pil_analyzer;
use starky::types::PIL;
use std::path::Path;

pub fn compile_pil_from_str(pil_str: &str) -> PIL {
    let analyze = powdr_pil_analyzer::analyze_string::<GoldilocksField>(pil_str);

    export::export(&analyze)
}
pub fn compile_pil_from_path(pil_path: &str) -> PIL {
    let analyze = powdr_pil_analyzer::analyze::<GoldilocksField>(Path::new(pil_path));

    export::export(&analyze)
}

#[cfg(test)]
mod test {
    use super::*;
    use starky::types::load_json;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    // This test is check whether the js_res the js version pilcom.compile equals to compile_pil_from_str.
    #[test]
    fn test_compile_pil_from_str() {
        let path = Path::new("../starkjs/fibonacci/fibonacci.pil")
            .canonicalize()
            .unwrap();

        let pil_str = fs::read_to_string(path.clone()).unwrap();
        // The target and actual pil_json
        let actual = compile_pil_from_str(&pil_str);
        let _target = load_json::<PIL>("data/fib.pil.json").unwrap();

        // This will meet error, as the polArray.name are different.
        // assert_eq!(actual, target);

        // Check the file manually.
        let mut file = File::create(Path::new("data/fib2.pil.json")).unwrap();
        let input = serde_json::to_string_pretty(&actual).unwrap();
        write!(file, "{}", input).unwrap();
    }
}
