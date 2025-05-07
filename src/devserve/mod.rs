use axum::{
    Router,
    routing::any_service,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_livereload::LiveReloadLayer;
use tracing::info;
use crate::config::Config;

use bareurl::BareUrlServeDir;

pub async fn run(config: &Config, livereload: LiveReloadLayer) -> anyhow::Result<()> {
    let website_dir = config.outdir.canonicalize()?;
    info!("serve website_dir: {}", &website_dir.display());

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 3456));

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

