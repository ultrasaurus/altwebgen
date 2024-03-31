use crate::config::Config;
use anyhow::{bail, Result};
use handlebars::Handlebars;
use std::{ffi::OsStr, fs::File, io::Read, path::Path};
use tracing::info;

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
pub fn render_file<P: AsRef<Path>>(
    config: &Config,
    hbs: &Handlebars,
    path: P,
) -> anyhow::Result<()> {
    let sourcepath = path.as_ref();
    info!("rendering: {}", sourcepath.display());
    let maybe_ext: Option<&str> = sourcepath.extension().and_then(OsStr::to_str);
    match maybe_ext {
        Some("hbs") => {
            // path for writing: w/o .hbs, rooted in output directory
            let writepath = config.outpath(sourcepath.with_extension(""))?;
            let writer = std::fs::File::options()
                .create(true)
                .write(true)
                .open(writepath)?;
            let source = read_file_to_string(sourcepath)?;
            let data: HashMap<String, String> = HashMap::new();
            hbs.render_template_to_write(&source, &data, writer)?;
        }
        Some("md") => {
            use pulldown_cmark as md;
            // use std::collections::HashMap;

            let mut html_body = Vec::new();

            let source = read_file_to_string(sourcepath)?;
            let parser = md::Parser::new_ext(&source,
                                md::Options::empty());
            md::html::write_html(&mut html_body, parser)?;

            // path for writing: html extension, rooted in output directory
            let writepath = config.outpath(sourcepath.with_extension("html"))?;
            let writer = std::fs::File::options()
                .create(true)
                .write(true)
                .open(writepath)?;

            let mut template_vars = HashMap::new();
            let body_string = String::from_utf8(html_body)?;
            template_vars.insert("body", body_string);

            hbs.render_to_write("default", &template_vars, writer)?;
        },
        _ => {
            // copy the file
            let _ = std::fs::copy(&path, config.outpath(&path)?);
        }
    }
    Ok(())
}
