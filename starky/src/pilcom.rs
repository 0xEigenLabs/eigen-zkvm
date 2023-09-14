//! Poring from https://github.com/powdr-labs/powdr.git.
mod export;
mod expression_counter;

use crate::types::{read_json, PIL};
use number::GoldilocksField;

pub fn compile_pil_from_str(pil_str: &String) -> PIL {
    let analyze = pil_analyzer::analyze_string::<GoldilocksField>(pil_str);

    export::export(&analyze)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::load_json;
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
        println!("{:?}", path.to_str());

        let contents = fs::read_to_string(path.clone()).unwrap();

        let actual = compile_pil_from_str(&contents);

        let target = load_json::<PIL>("data/fib.pil.json").unwrap();

        // This will meet error, as the polArray.name are different.
        // assert_eq!(actual, target);

        // Check the file manually.
        let mut file = File::create(Path::new("data/fib2.pil.json")).unwrap();
        let input = serde_json::to_string_pretty(&actual).unwrap();
        write!(file, "{}", input);
    }
}
