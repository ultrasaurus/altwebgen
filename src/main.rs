use tracing::info;
mod web;
mod config;
use config::Config;
mod devserve;
use walkdir::WalkDir;

fn get_current_working_dir() -> std::io::Result<std::path::PathBuf> {
    let wd = std::env::current_dir()?;
    info!("working directory: {}", wd.display());
    Ok(wd)
}

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
    let _wd = get_current_working_dir();
    let config:Config = Default::default();
    let result = process_files(&config);
    match result {
        Err(e) => println!("oops: {}", e),
        Ok(_) => {
            devserve::run(&config).await;
        }
    }
}

