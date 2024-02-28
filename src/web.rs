use anyhow::{Result, bail};
use std::fs::File;
use std::io::Read;
use tracing::info;

pub fn read_file_to_string(filename: &str) -> Result<String> {
    info!("read_file #{}", filename);
    let mut f = File::open(filename)?;
    let mut buf = String::new();
    let bytes = f.read_to_string(&mut buf)?;
    if bytes == 0 {
      bail!("failed to read: 0 bytes returned from read_to_string");
    }
    Ok(buf)
}

