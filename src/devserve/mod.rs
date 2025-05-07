use axum::{
    Router,
    routing::any_service,
};
use notify::{Config as WatchConfig, Watcher};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_livereload::LiveReloadLayer;
use tracing::{error, info};
use crate::{
    config::Config,
    setup
};

use bareurl::BareUrlServeDir;

pub async fn run(config: &Config) -> anyhow::Result<()> {
    let website_dir = config.outdir.canonicalize()?;

    info!("serve website_dir: {}", &website_dir.display());

    // initial build
    setup::clean_build(&config)?;

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 3456));

    // setup live reload to watch files and rebuild when changed
    let livereload = LiveReloadLayer::new();
    let reloader = livereload.reloader();

    let config_watcher_copy = config.clone();
    let mut watcher = notify::recommended_watcher(move |_|
        if let Err(e) = setup::clean_build(&config_watcher_copy) {
            error!("change detected, then build failed: {:?}", e);
        } else {
            info!("change detected, build complete\n---\n");
        reloader.reload()
        }
    )?;
    // TODO: if build takes more than 2 sec, then this will loop
    watcher.configure(WatchConfig::default().with_poll_interval(Duration::from_secs(2)))?;
    watcher.watch(&website_dir, notify::RecursiveMode::Recursive)?;


    let app = Router::new()
        .fallback(any_service(BareUrlServeDir::new(&website_dir)))
        .layer(livereload);

    let listener = TcpListener::bind(addr).await?;

    axum::serve(
        listener,
        app,
    ).await?;

    Ok(())
}

