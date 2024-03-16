use anyhow::{Result, bail};
use handlebars::Handlebars;
use std::{fs::File, ffi::OsStr, io::Read, path::Path};
use tracing::info;
use crate::config::Config;

pub fn read_file_to_string<P: AsRef<Path>>(filepath: P) -> Result<String> {
    let path = std::fs::canonicalize(filepath)?;
    info!("read_file #{}", path.display());
    let mut f = File::open(path)?;
    let mut buf = String::new();
    let bytes = f.read_to_string(&mut buf)?;
    if bytes == 0 {
      bail!("failed to read: 0 bytes returned from read_to_string");
    }
    Ok(buf)
}

use std::collections::HashMap;
pub fn render_file<P: AsRef<Path>>(config: &Config, hbs: &Handlebars, path: P) -> anyhow::Result<()> {
   let sourcepath = path.as_ref();
   let maybe_ext: Option<&str> = sourcepath.extension().and_then(OsStr::to_str);
   if let Some(ext) = maybe_ext {
        if ext == "erb" {
            //anyhow::bail!("erb: not yet supported");
            // path for writing: w/o .erb, rooted in output directory
            let writepath = config.outpath(sourcepath.with_extension(""))?;
            let writer = std::fs::File::open(writepath)?;
            let sourcefile = read_file_to_string(sourcepath)?;
            let data: HashMap<String, String> = HashMap::new();
            hbs.render_template_to_write(&sourcefile,
                    &data,
                    writer)?;
        } else {
            // copy the file
            let _ = std::fs::copy(&path, config.outpath(&path)?);
        }
   }
   Ok(())
}
