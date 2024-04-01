use handlebars::Handlebars;
use std::{default::Default, path::{Path, PathBuf}};
use tracing::{info, error};
use anyhow::anyhow;
use walkdir::WalkDir;
use warp::ws::Message;

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

// copy a directory with all of its files recursively
fn copy_dir_all<P: AsRef<Path>>(src: P, dst: &Path) -> anyhow::Result<()> {
    let dst_path: &Path = dst.as_ref();
    let dst_dir = dst_path.to_path_buf();
    for entry_result in WalkDir::new(&src) {
        let entry = entry_result.map_err(|_| {
            anyhow!("failed to copy directory, from {} to {}",
                src.as_ref().display(),
                dst_path.display())
        })?;

        let from = entry.path();
        let to = dst_dir.join(from.strip_prefix(&src)?);
        println!("\tcopy {} => {}", from.display(), to.display());

        // create directories
        if entry.file_type().is_dir() {
            if let Err(e) = std::fs::create_dir(to) {
                match e.kind() {
                    std::io::ErrorKind::AlreadyExists => {}
                    _ => return Err(e.into()),
                }
            }
        }
        // copy files
        else if entry.file_type().is_file() {
            std::fs::copy(&from, &to).map_err(|_| {
                anyhow!("copy_dir_all: failed to copy file, from {} to {}",
                    from.display(),
                    to.display())
            })?;
        }
        // ignore the rest
        else {
            eprintln!("copy: ignored symlink {}", from.display());
        }
    }
    Ok(())
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

fn clean_and_recreate_dir<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
    let path_ref = path.as_ref();
    if path_ref.exists() {
        std::fs::remove_dir_all(path_ref).map_err(|e| {
            anyhow!(format!("failed to delete directory: {}, error: {}", path_ref.display(), e))
        })?;
    }
    std::fs::create_dir_all(path_ref).map_err(|e| {
            anyhow!(format!("failed to create directory: {}, error: {}", path_ref.display(), e))
        })?;
    Ok(())
}


fn process_files(config: &Config, handlebars: &Handlebars) -> anyhow::Result<()> {
    info!("Processing files...");
    clean_and_recreate_dir(&config.outdir)?;

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

fn setup_templates(config: &Config, hbs: &mut Handlebars) -> anyhow::Result<()> {
    info!("setup_templates");
    clean_and_recreate_dir(&config.builddir)?;
    let buildtemplatedir = config.buildtemplatedir();
    copy_dir_all(&config.templatedir, &buildtemplatedir)?;
    copy_dir_all("ref", &buildtemplatedir.join("ref"))?;

        let buildtemplatedir = config.buildtemplatedir();
    hbs.register_templates_directory(&buildtemplatedir, Default::default())
        .map_err(|_| {
            anyhow!("failed to register template directory: {}", buildtemplatedir.display())
        })?;
    info!("Setup: template directory '{}' registered", &buildtemplatedir.display());

    Ok(())
}
fn setup() -> anyhow::Result<(Config, Handlebars<'static>)> {
    info!("Setup: start");
    info!("       working directory {}", get_current_working_dir()?.display());
    let config:Config = Default::default();
    let mut hbs = Handlebars::new();
    setup_templates(&config, &mut hbs)?;
    process_files(&config, &hbs)?;
    info!("Setup: complete");
    Ok((config, hbs))
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // install global subscriber configured based on RUST_LOG envvar.
    tracing_subscriber::fmt::init();
    info!("Logging enabled");

    let (config, mut hbs) = setup()?;
    let mut source_watch = Vec::new();
    source_watch.push(config.sourcedir.clone());
    let mut template_watch = Vec::new();
    template_watch.push(config.templatedir.clone());
    template_watch.push(PathBuf::from("ref"));

    // A channel used to broadcast to any websockets to reload when a file changes.
    let (tx, _rx) = tokio::sync::broadcast::channel::<Message>(100);
    loop {
        tokio::select! {
            _ = devserve::run(&config, tx.clone()) => {
                error!("unexpected server end");
                break
            },
            source_result = watch(&source_watch) => {
                info!("source watcher result {:?}", source_result);
                if let Err(e) = process_files(&config, &hbs) {
                        error!("process_files failed: {:?}", e);
                        break
                } else {
                    let _ = tx.send(Message::text("reload"));
                }
            },
            template_result = watch(&template_watch) => {
                info!("template watcher result {:?}", template_result);
                hbs.clear_templates();
                if let Err(e) = setup_templates(&config, &mut hbs) {
                    error!("setup_templates failed: {:?}", e);
                    break
                };
                if let Err(e) = process_files(&config, &hbs) {
                        error!("process_files failed: {:?}", e);
                        break
                } else {
                    let _ = tx.send(Message::text("reload"));
                }
            }
        }
    }
    Ok(())

}


