use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};

pub fn write_vec_to_file(path: &str, vec: &[u64]) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    for num in vec {
        write!(file, "{}\n", num)?;
    }
    Ok(())
}

pub fn read_vec_from_file(path: &str) -> std::io::Result<Vec<u64>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut vec = Vec::new();
    for line in reader.lines() {
        let num: u64 = line.unwrap().parse().unwrap();
        vec.push(num);
    }
    Ok(vec)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_read_write_vec_with_file() -> std::io::Result<()> {
        let target: Vec<u64> = vec![1, 2, 3, 4, 5];
        let path = String::from("./vec_data.txt");
        write_vec_to_file(&path, &target)?;

        let actual = read_vec_from_file(&path)?;
        assert_eq!(actual, target);

        Ok(())
    }
}
