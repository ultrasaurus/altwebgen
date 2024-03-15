use tracing::{info, error};
mod web;
mod util;
mod config;
use config::Config;
mod devserve;
use walkdir::WalkDir;
use crate::util::*;

fn get_current_working_dir() -> std::io::Result<std::path::PathBuf> {
    let wd = std::env::current_dir()?;
    info!("working directory: {}", wd.display());
    Ok(wd)
}

fn create_destdir(config: &Config, sourcepath: &std::path::Path) -> anyhow::Result<()> {
    let rel_path = sourcepath
        .strip_prefix(&config.sourcedir);
    if rel_path.is_err() {
        anyhow::bail!("expected strip prefix match for soucepath {} and sourcedir {}", 
            sourcepath.display(), config.sourcedir.display());
    } else {
        let dest_path = config.outdir.join(rel_path?);
        let result = std::fs::create_dir_all(&dest_path);
        if result.is_err() {
            anyhow::bail!("failed to create {}", &dest_path.display())
        } 
    }
    Ok(())
}
fn process_files(config: &Config) -> anyhow::Result<()> {
    let walker = WalkDir::new(&config.sourcedir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            !e.is_hidden()
        });

    for entry_result in walker
    {
        let entry = entry_result?;
        let path = entry.path();
        if path.is_dir() {
            create_destdir(config, path)?;
        } else {
            web::render_file(config, path)?;
        }
    }

   Ok(())
}

#[tokio::main]
async fn main() {
    // install global subscriber configured based on RUST_LOG envvar.
    tracing_subscriber::fmt::init();
    info!("Logging enabled");
    let _wd = get_current_working_dir();
    let config:Config = Default::default();
    let result = process_files(&config);
    match result {
        Err(e) => println!("oops: {}", e),
        Ok(_) => {
            devserve::run(&config).await;
        }
    }
}

