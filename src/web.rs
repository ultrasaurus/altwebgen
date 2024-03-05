use anyhow::{Result, bail};
use std::{fs::File, ffi::OsStr, io::Read, path::Path};
use tracing::info;
use crate::config::Config;

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

pub fn render_file<P: AsRef<Path>>(config: &Config, path: P) -> anyhow::Result<()> {
   let sourcepath = path.as_ref();
   let maybe_ext: Option<&str> = sourcepath.extension().and_then(OsStr::to_str);
   if let Some(ext) = maybe_ext {
        if ext == "erb" {
            anyhow::bail!("erb: not yet supported");
            // path for writing: w/o .erb, rooted in output directory
            // let writepath = config.outpath(sourcepath.with_extension(""))?;
            // process erb
        } else {
            // copy the file
            let _ = std::fs::copy(&path, config.outpath(&path)?);
        }
   }
   Ok(())
}
