use anyhow::{Context, Error};
use serde::de::DeserializeOwned;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

pub fn create_file<P: AsRef<Path>>(path: P) -> Result<BufWriter<File>, Error> {
    let path = path.as_ref();
    fs::create_dir_all(path.parent().context("no parent")?)?;
    Ok(BufWriter::new(File::create(path)?))
}

pub fn read_json<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T, Error> {
    let path = path.as_ref();
    let reader = BufReader::new(File::open(path)?);
    Ok(serde_json::from_reader(reader)?)
}
