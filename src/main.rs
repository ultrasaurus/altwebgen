
use tracing::info;

mod config;
use config::Config;
mod devserve;
mod setup;

mod util;
mod watch;
mod web;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // install global subscriber configured based on RUST_LOG envvar.
    tracing_subscriber::fmt::init();
    info!("Logging enabled");

    let config:Config = Default::default();

    watch::run(&config).await?;

    Ok(())
}


