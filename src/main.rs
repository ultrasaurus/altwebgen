use tracing::info;
mod web;
mod config;
use config::Config;
mod devserve;
use walkdir::WalkDir;

fn process_files(config: &Config) -> anyhow::Result<()> {
   for entry in WalkDir::new(&config.sourcedir) {
        web::render_file(config, entry?.path())?;
   }
   Ok(())
}

#[tokio::main]
async fn main() {
    // install global subscriber configured based on RUST_LOG envvar.
    tracing_subscriber::fmt::init();
    info!("Logging enabled");

    let config:Config = Default::default();
    let result = process_files(&config);
    match result {
        Err(e) => println!("oops: {}", e),
        Ok(_) => {
            devserve::run(&config).await;
        }
    }
}

