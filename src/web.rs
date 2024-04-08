use crate::config::Config;
use anyhow::{bail, Result};
use handlebars::Handlebars;
use pulldown_cmark as md;
use std::{ffi::OsStr, fs::File, io::Read, path::Path};
use std::collections::HashMap;
use tracing::{info, trace};

pub fn read_file_to_string<P: AsRef<Path>>(filepath: P) -> Result<String> {
    let path = std::fs::canonicalize(filepath)?;
    trace!("read_file #{}", path.display());
    let mut f = File::open(path)?;
    let mut buf = String::new();
    let bytes = f.read_to_string(&mut buf)?;
    if bytes == 0 {
        bail!("failed to read: 0 bytes returned from read_to_string");
    }
    Ok(buf)
}

pub fn md2html<P: AsRef<Path>>(sourcepath: P) -> Result<Vec<u8>> {
    let mut html_body = Vec::new();

    let source = read_file_to_string(sourcepath)?;
    let parser = md::Parser::new(&source);
    md::html::write_html(&mut html_body, parser)?;

    Ok(html_body)
}

fn read_source<P: AsRef<Path>>(sourcepath: P) -> Result<(HashMap<String, String>, String)>
{
    let source = read_file_to_string(sourcepath)?;
    use matter::matter;
    let (data, content) = match matter(&source) {
        None => {info!("matter: None");
            let data: HashMap<String, String> = HashMap::new();
            (data, source)
        },

        Some((yaml_string, content)) => {
            info!("matter:\n{:?}\n------", yaml_string);
            let data:HashMap<String, String> = serde_yaml::from_str(&yaml_string)?;

            //  let data: HashMap<&str, String> = HashMap::new();
            (data, content)
        }
    };
    Ok((data, content))
}

pub fn render_file<P: AsRef<Path>>(
    config: &Config,
    hbs: &Handlebars,
    path: P,
) -> anyhow::Result<()> {
    let sourcepath = path.as_ref();
    trace!("rendering: {}", sourcepath.display());
    let maybe_ext: Option<&str> = sourcepath.extension().and_then(OsStr::to_str);
    match maybe_ext {
        Some("hbs") => {
            let (mut data, content) = read_source(sourcepath)?;
            // path for writing: w/o .hbs, rooted in output directory
            let writepath = config.outpath(sourcepath.with_extension(""))?;
            let writer = std::fs::File::options()
                .create(true)
                .write(true)
                .open(writepath)?;
            let html_body = hbs.render_template(&content, &data)?;
            data.insert("body".into(), html_body);
            // hbs.render_template_to_write("default", &data, writer)?;
            hbs.render_to_write("default", &data, writer)?;

        }
        Some("md") => {
            let html_body = md2html(sourcepath)?;

            // let source = read_file_to_string(sourcepath)?;
            // let parser = md::Parser::new(&source);
            // md::html::write_html(&mut html_body, parser)?;

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
