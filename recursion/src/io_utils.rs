use std::fs::File;
use std::io::Write;

pub fn write_vec_to_file(path: &str, vec: &[u64]) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    let input = serde_json::to_string(&vec)?;
    write!(file, "{}", input)?;
    Ok(())
}

pub fn read_vec_from_file(input_file: &str) -> std::io::Result<Vec<u64>> {
    let inputs_str = std::fs::read_to_string(input_file)?;
    let output: Vec<u64> = serde_json::from_str(&inputs_str)?;
    Ok(output)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_read_write_vec_with_file() -> std::io::Result<()> {
        let target: Vec<u64> = vec![1, 2, 3, 4, 5, 1111112121, 2667022304383014929];
        let path = String::from("/tmp/vec_data.txt");
        write_vec_to_file(&path, &target)?;

        let actual = read_vec_from_file(&path)?;
        assert_eq!(actual, target);

        Ok(())
    }
}
