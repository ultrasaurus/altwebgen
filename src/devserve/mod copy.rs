use std::net::{SocketAddr, ToSocketAddrs};
use std::path::PathBuf;
use tokio::sync::broadcast;
use tracing::info;
use warp::Filter;
use warp::ws::Message;
use crate::config::Config;

/// The HTTP endpoint for the websocket used to trigger reloads when a file changes.
const LIVE_RELOAD_ENDPOINT: &str = "__livereload";

async fn serve(website_dir: PathBuf,
                address: SocketAddr,
                reload_tx: broadcast::Sender<Message>
) -> anyhow::Result<()> {
    info!("devserve::serve website_dir: {}", website_dir.display());

    //---------------------- Serve Website Files ------------------------------
    let root = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file(website_dir.join("index.html")));

    // dir already requires GET...
    let all = warp::fs::dir(website_dir);

    // GET / => index.html
    // GET /... => ./examples/..
    let routes = root.or(all)
                .with(warp::trace::request());

    //---------------------- Start server ------------------------------
    warp::serve(routes).run(([127, 0, 0, 1], 3456)).await;


    Ok(())
}

pub async fn run(config: &Config) -> anyhow::Result<()> {
    info!("devserve::run");
    let port = 3456;
    let address: String = format!("{}:{}", "localhost", port);

    let sockaddr: SocketAddr = address
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow::anyhow!("no address found for {}", address))?;

            // A channel used to broadcast to any websockets to reload when a file changes.
    let (tx, _rx) = tokio::sync::broadcast::channel::<Message>(100);

    let reload_tx = tx.clone();
    // let thread_handle = std::thread::spawn(move || {
    //     serve(website_dir, sockaddr, reload_tx);
    // });
    let website_dir: std::path::PathBuf = config.outdir.canonicalize()?;
    let serve_handle = serve(website_dir, sockaddr, reload_tx);

    let serving_url = format!("http://{}", address);
    info!("Serving on: {}", serving_url);

    // if open_browser {
    //     open(serving_url);
    // }
    Ok(())
}