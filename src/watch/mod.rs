
use std::path::PathBuf;
use tracing::{info, error};
use warp::ws::Message;

use crate::{
    devserve,
    config::Config,
    setup,
    util::*,
    web
};

mod watch_files;
use watch_files::detect_changes;

pub async fn run(config: &Config) -> anyhow::Result<()> {
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
            source_result = detect_changes(&source_watch) => {
                info!("source watcher result {:?}", source_result);
                clean_and_recreate_dir(&config.outdir)?;
                if let Err(e) = web::process_files(&config, &hbs) {
                        error!("process_files failed: {:?}", e);
                        break
                } else {
                    let _ = tx.send(Message::text("reload"));
                }
            },
            template_result = detect_changes(&template_watch) => {
                info!("template watcher result {:?}", template_result);
                hbs.clear_templates();
                if let Err(e) = setup::clean_build(&config, &mut hbs) {
                    error!("build failed: {:?}", e);
                    break
                } else {
                    let _ = tx.send(Message::text("reload"));
                }
            }
        }
    }


    Ok(())
}