use std::path::Path;
use tracing::{info, trace};
use walkdir::WalkDir;

mod audio;
mod document;
use document::Document;
mod framework;
use framework::render_html;

pub mod md;
pub use md::Ref as Ref;

pub mod words;

use crate::{
    config::Context,
    util::*,
};



fn build_source_files(context: &Context) -> anyhow::Result<()> {
    let config: &crate::config::Config = context.config;
    info!("Bulding source files...");
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
            create_destdir(&config.sourcedir, &config.outdir, path)?;
        } else {
            render_file(context, path)?;
        }
    }
    Ok(())
}

fn copy_template_assets(context: &Context) -> anyhow::Result<()> {
    let config: &crate::config::Config = context.config;
    info!("Copying template assets...");
    let walker = WalkDir::new(&config.templatedir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            !e.is_hidden()
        });

    for entry_result in walker
    {
        // should be declared somewhere central web::template_extension ?
        let template_extension =std::ffi::OsStr::new("hbs");

        trace!("  entry: {:?}", entry_result);
        let entry = entry_result?;
        let path = entry.path();
        if path.is_dir() {
            create_destdir(&config.templatedir, &config.outdir, path)?;
        } else if path.extension() != Some(template_extension) {
            std::fs::copy(&path, config.templatedir_outpath(&path)?)?;
        }
    }

    Ok(())
}



pub fn process_files(context: &Context) -> anyhow::Result<()> {
   build_source_files(&context)?;
   copy_template_assets(&context)?;
   Ok(())
}



fn render_file<P: AsRef<Path>>(
    context: &Context,
    path: P,
) -> anyhow::Result<()> {
    let config = context.config;
    let sourcepath = path.as_ref();
    trace!("render_file: {}", sourcepath.display());
    let document = Document::from_path(&path);
    match document.html_generator(&context)? {
        None => { std::fs::copy(&path, config.outpath(&path)?)?;},
        Some(html_source) => {
            let writepath = document.outpath(config)?;
            let mut writer = std::fs::File::options()
                .create(true)
                .write(true)
                .open(writepath)?;
            render_html(&context, html_source, &mut writer)?;
        }

    }
    Ok(())
}
