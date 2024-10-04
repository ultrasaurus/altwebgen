use anyhow::Result;
use handlebars::Handlebars;
use std::{ffi::OsStr, path::Path};
use std::collections::HashMap;
use tracing::{info, trace};
use walkdir::WalkDir;

mod md;
pub use md::Ref as Ref;

use crate::{
    config::Config,
    util::*,
};


pub fn process_files(config: &Config, handlebars: &Handlebars) -> anyhow::Result<()> {
    info!("Processing files...");
    let walker = WalkDir::new(&config.sourcedir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            !e.is_hidden()
        });

    for entry_result in walker
    {
        trace!("  entry: {:?}", entry_result);
        let entry = entry_result?;
        let path = entry.path();
        if path.is_dir() {
            create_destdir(config, path)?;
        } else {
            render_file(config, &handlebars, path)?;
        }
    }

   Ok(())
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

fn render_file<P: AsRef<Path>>(
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
            let site_attr_ref = &config.site_attr;
            data.extend(site_attr_ref.into_iter().map(|(k, v)| (k.clone(), v.clone())));

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
            let html_body = md::file2html(sourcepath)?;

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
