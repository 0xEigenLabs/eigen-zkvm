use anyhow::Result;

pub fn write_vec_to_file<W: std::io::Write>(writer: &mut W, vec: &[u64]) -> Result<()> {
    let input = serde_json::to_string(&vec)?;
    write!(writer, "{input}")?;
    Ok(())
}

pub fn read_vec_from_file<R: std::io::Read>(reader: R) -> Result<Vec<u64>> {
    let output = serde_json::from_reader(reader)?;
    Ok(output)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_read_write_vec_with_file() -> Result<()> {
        let target: Vec<u64> = vec![1, 2, 3, 4, 5, 1111112121, 2667022304383014929];
        let path = String::from("/tmp/vec_data.txt");
        let mut file = std::fs::File::create(path.clone()).unwrap();
        write_vec_to_file(&mut file, &target)?;

        let file = std::fs::File::open(path).unwrap();
        let actual = read_vec_from_file(file)?;
        assert_eq!(actual, target);

        Ok(())
    }
}
