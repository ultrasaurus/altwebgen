
use std::path::PathBuf;
use tracing::{info, error};
use tower_livereload::LiveReloadLayer;

use crate::{
    config::Config,
    devserve,
    setup,
    util::*,
    web
};

mod watch_files;
use watch_files::detect_changes;


pub async fn run(config: &Config) -> anyhow::Result<()> {
    let mut context= setup::init_and_build(&config)?;
    let mut source_watch = Vec::new();
    source_watch.push(config.sourcedir.clone());
    let mut template_watch = Vec::new();
    template_watch.push(config.templatedir.clone());
    template_watch.push(PathBuf::from("ref"));

    // Tower Layer to handle browser/client comms
    let livereload = LiveReloadLayer::new();
    let reloader = livereload.reloader();

    tokio::select! {
        _ = devserve::run(&config, livereload) => {
            println!("server shutdown")
        }
        _ = async {
            loop {
                info!("watching for changes...");
                tokio::select! {
                    source_result = detect_changes(&source_watch) => {
                        info!("source watcher result {:?}", source_result);
                        clean_and_recreate_dir(&config.outdir)?;
                        if let Err(e) = web::process_files(&context) {
                            error!("process_files failed: {:?}", e);
                        } else {
                            reloader.reload();
                        }
                    },
                    template_result = detect_changes(&template_watch) => {
                        info!("template watcher result {:?}", template_result);
                        match setup::clean_build(&config) {
                            Err(e) => {error!("build failed: {:?}", e); break},
                            Ok(new_context) => {
                                context = new_context;
                                reloader.reload();
                            }
                        }
                    }
                }
            }
            anyhow::Result::<()>::Ok(())
        } => {}

    };

    Ok(())
}

