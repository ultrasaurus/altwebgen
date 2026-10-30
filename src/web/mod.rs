use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::{info, trace};
use walkdir::WalkDir;

mod audio;
pub mod collection;
use collection::Collections;
pub mod date;
mod document;
use document::Document;
mod framework;
use framework::render_html;

pub mod md;
pub use md::Ref;

pub mod people;

mod series;

pub mod template;

pub mod words;

use crate::{config::Context, util::*};

// output paths written (or found already up to date) during one build
type Outputs = HashSet<PathBuf>;

fn build_source_files(context: &Context, outputs: &mut Outputs) -> anyhow::Result<()> {
    let config: &crate::config::Config = context.config;
    let collections = Collections::load(config)?;
    info!("Bulding source files...");
    let walker = WalkDir::new(&config.sourcedir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !e.is_hidden());

    for entry_result in walker {
        trace!("  entry: {:?}", entry_result);
        let entry = entry_result?;
        let path = entry.path();
        if path.is_dir() {
            create_destdir(&config.sourcedir, &config.outdir, path)?;
            outputs.insert(config.outpath(path)?);
        } else if is_config_file(path) || collections.is_draft(path) {
            trace!("  not published: {}", path.display());
        } else {
            outputs.insert(render_file(context, &collections, path)?);
        }
    }
    Ok(())
}

fn is_config_file(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| collection::CONFIG_FILES.iter().any(|f| name == *f))
}

fn copy_template_assets(context: &Context, outputs: &mut Outputs) -> anyhow::Result<()> {
    let config: &crate::config::Config = context.config;
    info!("Copying template assets...");
    let walker = WalkDir::new(&config.templatedir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !e.is_hidden());

    for entry_result in walker {
        // should be declared somewhere central web::template_extension ?
        let template_extension = std::ffi::OsStr::new("hbs");

        trace!("  entry: {:?}", entry_result);
        let entry = entry_result?;
        let path = entry.path();
        if path.is_dir() {
            create_destdir(&config.templatedir, &config.outdir, path)?;
            outputs.insert(config.templatedir_outpath(path)?);
        } else if path.extension() != Some(template_extension) {
            let outpath = config.templatedir_outpath(path)?;
            copy_if_changed(path, &outpath)?;
            outputs.insert(outpath);
        }
    }

    Ok(())
}

// builds into the output directory in place: unchanged files aren't
// rewritten, and anything this build didn't produce is removed
pub fn process_files(context: &Context) -> anyhow::Result<()> {
    let config = context.config;
    ensure_outdir(&config.outdir)?;
    let mut outputs: Outputs = context.ref_outputs.iter().cloned().collect();
    outputs.insert(config.outdir.join("media"));
    build_source_files(context, &mut outputs)?;
    copy_template_assets(context, &mut outputs)?;
    let removed = remove_stale(&config.outdir, &outputs)?;
    if removed > 0 {
        info!("removed {removed} stale files and folders from the output");
    }
    Ok(())
}

// renders or copies one source file; returns where it was written
fn render_file<P: AsRef<Path>>(
    context: &Context,
    collections: &Collections,
    path: P,
) -> anyhow::Result<PathBuf> {
    let config = context.config;
    let sourcepath = path.as_ref();
    trace!("render_file: {}", sourcepath.display());
    let document = Document::from_path(&path);
    match document.html_generator(context, collections)? {
        None => {
            let outpath = config.outpath(&path)?;
            copy_if_changed(sourcepath, &outpath)?;
            Ok(outpath)
        }
        Some(html_source) => {
            let writepath = match collections.outpath(sourcepath) {
                Some(outpath) => {
                    outpath.create_all_parent_dir()?;
                    outpath.to_path_buf()
                }
                None => document.outpath(config)?,
            };
            let mut html: Vec<u8> = Vec::new();
            render_html(context, html_source, &mut html)?;
            write_if_changed(&writepath, &html)?;
            Ok(writepath)
        }
    }
}
