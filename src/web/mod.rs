use std::path::Path;
use tracing::{info, trace};
use walkdir::WalkDir;
mod document;
use document::Document;

pub mod md;
pub use md::Ref as Ref;

mod audio;
pub mod words;

use crate::{
    config::Context,
    util::*,
};


pub fn process_files(context: &Context) -> anyhow::Result<()> {
    let config = context.config;
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
            render_file(context, path)?;
        }
    }

   Ok(())
}


fn render_file<P: AsRef<Path>>(
    context: &Context,
    path: P,
) -> anyhow::Result<()> {
    let config = context.config;
    let sourcepath = path.as_ref();
    trace!("rendering: {}", sourcepath.display());
    let document = Document::from_path(&path);
    match document.html_generator(&context)? {
        None => { std::fs::copy(&path, config.outpath(&path)?)?;},
        Some(html_source) => {
            let writepath = document.outpath(config)?;
            let mut writer = std::fs::File::options()
                .create(true)
                .write(true)
                .open(writepath)?;
            html_source.render(&context, &mut writer)?;
        }

    }
    Ok(())
}
