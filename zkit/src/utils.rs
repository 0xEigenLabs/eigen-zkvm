use std::fmt::write;
use std::fs::File;
use std::io::{Write, Read, BufWriter};
use anyhow::{anyhow, Result};
use std::path::Path;
use std::boxed::Box;

pub fn new_writer(pathname: &str) -> Result<BufWriter<Box<dyn Write>>> {
    let path = Path::new(pathname);
    assert!(
        !path.exists(),
        "file exists: {}",
        path.display()
    );

    let mut buffer = File::create(pathname)?;
    Ok(BufWriter::new(Box::new(buffer)))
}