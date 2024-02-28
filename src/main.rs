
use tracing::{info};
mod web;
mod devserve;

#[tokio::main]
async fn main() {
    // install global subscriber configured based on RUST_LOG envvar.
    tracing_subscriber::fmt::init();
    info!("Logging enabled");

    devserve::run().await;
}

