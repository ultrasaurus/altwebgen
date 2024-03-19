use std::path::{Path, PathBuf};
use tokio::sync::broadcast;
use tracing::info;
use warp::Filter;
use warp::ws::Message;
use crate::config::Config;

mod ws;
use ws::ws_receiver;

/// The HTTP endpoint for the websocket used to trigger reloads when a file changes.
const LIVE_RELOAD_ENDPOINT: &str = "__livereload";

pub async fn serve<P: AsRef<Path>>(path: P, from_server_tx: broadcast::Sender<Message>) -> anyhow::Result<()> {
    info!("serve website_dir: {}", path.as_ref().display());
    let website_dir = PathBuf::from(path.as_ref());
    let index_path = website_dir.join("index.html");
     // GET / => index.html
    let root = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file(index_path));

    // dir already requires GET...
    let all = warp::fs::dir(website_dir);
    let livereload = ws_receiver(LIVE_RELOAD_ENDPOINT, from_server_tx);
    let http_routes = root.or(all)
                .with(warp::trace::request());

    let routes = livereload.or(http_routes);
    warp::serve(routes).run(([127, 0, 0, 1], 3456)).await;
    Ok(())
}


pub async fn run(config: &Config,
                reload_tx: tokio::sync::broadcast::Sender<Message>
) -> anyhow::Result<()> {
    info!("devserve::run");
    let website_dir = config.outdir.canonicalize()?;

    serve(&website_dir, reload_tx).await
}
