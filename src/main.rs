use anyhow::anyhow;
use handlebars::Handlebars;
use std::{default::Default, path::{Path, PathBuf}};
use tracing::{info, warn, error};
use walkdir::WalkDir;

mod web;
mod util;
mod config;
use config::Config;
mod devserve;
use util::*;

mod watch;
use watch::watch;

fn get_current_working_dir() -> std::io::Result<PathBuf> {
    let wd = std::env::current_dir()?;
    info!("working directory: {}", wd.display());
    Ok(wd)
}

fn create_destdir(config: &Config, sourcepath: &Path) -> anyhow::Result<()> {
    let rel_path = sourcepath
        .strip_prefix(&config.sourcedir);
    if rel_path.is_err() {
        let err_report = format!("expected strip prefix match for soucepath {} and sourcedir {}",
            sourcepath.display(), config.sourcedir.display());
        error!(err_report);
        anyhow::bail!(err_report);
    } else {
        let dest_path = config.outdir.join(rel_path?);
        let result = std::fs::create_dir_all(&dest_path);
        if result.is_err() {
            anyhow::bail!("failed to create {}", &dest_path.display())
        }
    }
    Ok(())
}

fn clean_and_setup_outpath<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
    if path.as_ref().exists() {
        std::fs::remove_dir_all(&path)?;
    }
    std::fs::create_dir_all(&path)?;
    Ok(())
}


fn process_files(config: &Config, handlebars: &Handlebars) -> anyhow::Result<()> {
    info!("Processing files...");
    clean_and_setup_outpath(&config.outdir)?;
    let walker = WalkDir::new(&config.sourcedir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            !e.is_hidden()
        });

    for entry_result in walker
    {
        info!("  entry: {:?}", entry_result);
        let entry = entry_result?;
        let path = entry.path();
        if path.is_dir() {
            create_destdir(config, path)?;
        } else {
            web::render_file(config, &handlebars, path)?;
        }
    }

   Ok(())
}

fn setup(config: &Config, hbs: &mut Handlebars) -> anyhow::Result<()> {
    info!("Setup: start");
    hbs.register_templates_directory("templates", Default::default())?;
    info!("Setup: template directory '{}' registered", "templates");
    process_files(&config, &hbs)?;
    info!("Setup: complete");
    Ok(())
}


#[tokio::main]
async fn main() {
    // install global subscriber configured based on RUST_LOG envvar.
    tracing_subscriber::fmt::init();
    info!("Logging enabled");
    let _wd = get_current_working_dir();
    let config:Config = Default::default();
    let mut hbs = Handlebars::new();
    if let Err(e) = setup(&config, &mut hbs) {
        println!("oops, setup failed: {}", e);
    } else {
        let watch_dir = config.sourcedir.clone();
        loop {
            tokio::select! {
                _ = devserve::run(&config) => {
                    error!("unexpected server end");
                    break
                },
                watch_result = watch(&watch_dir) => {
                    info!("watcher result {:?}", watch_result);
                    if let Err(e) = process_files(&config, &hbs) {
                         error!("process_files failed: {:?}", e);
                         break
                    }
                }
            }
        }
    };

}


