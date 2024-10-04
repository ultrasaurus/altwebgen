
use anyhow;
use tracing::info;

mod config;
use config::Config;
mod devserve;
mod setup;

mod util;
mod watch;
mod web;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// directory path for markdown source files
    #[clap(short, long, value_parser, default_value = "source")]
    indir: String,

    /// destination path for html
    #[clap(short, long, value_parser, default_value = ".dist")]
    outdir: String,

    /// directory path for template files
    #[clap(short, long, value_parser, default_value = "templates")]
    templatedir: String,

    #[command(subcommand)]
    command: Option<Command>,

}

#[derive(Subcommand, Debug)]
enum Command {
    Dev,
    Build
}


fn cli_config(cli: &Cli) -> Config {
    Config::new(&*cli.outdir,
            &*cli.indir,
            &*cli.templatedir)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // install global subscriber configured based on RUST_LOG envvar.
    tracing_subscriber::fmt::init();
    info!("Logging enabled");

    let cli = Cli::parse();
    let config:Config = cli_config(&cli);
    match cli.command {
        None => println!("\nuse command 'dev' for  watch server or 'build' for generating static files\n\n"),
        Some(cmd) => match cmd {
            Command::Dev => watch::run(&config).await?,
            Command::Build => {
                let _hbs = setup::init(&config)?;
            }
        }
    };



    Ok(())
}


