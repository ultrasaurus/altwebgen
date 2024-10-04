use std::path::PathBuf;

use tracing::{info, error};
use warp::ws::Message;

mod config;
use config::Config;
mod devserve;
mod setup;

mod util;
use util::*;

mod watch;
use watch::watch;

mod web;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // install global subscriber configured based on RUST_LOG envvar.
    tracing_subscriber::fmt::init();
    info!("Logging enabled");

    let config:Config = Default::default();
    let  mut hbs= setup::init(&config)?;

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
                clean_and_recreate_dir(&config.outdir)?;
                if let Err(e) = web::process_files(&config, &hbs) {
                        error!("process_files failed: {:?}", e);
                        break
                } else {
                    let _ = tx.send(Message::text("reload"));
                }
            },
            template_result = watch(&template_watch) => {
                info!("template watcher result {:?}", template_result);
                hbs.clear_templates();
                clean_and_recreate_dir(&config.outdir)?;
                if let Err(e) = setup::init_templates(&config, &mut hbs) {
                    error!("setting up templates failed: {:?}", e);
                    break
                };
                if let Err(e) = web::process_files(&config, &hbs) {
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


