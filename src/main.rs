
use anyhow;
use tracing::info;

mod config;
use config::*;
mod devserve;
mod setup;

mod util;
mod watch;
mod web;

use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Command {
    Dev,
    Build
}

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
    #[clap(short, long, value_parser, default_value = "template")]
    templatedir: String,

    /// path prefix: change if deploying somewhere that is not root path
    #[clap(short, long, value_parser, default_value = "")]
    prefix: String,

    #[command(subcommand)]
    command: Option<Command>,

}

fn cli_config(cli: &Cli) -> Config {
    assert!(cli.command.is_some()); // programmer error, UI should enforce
    let mode = match cli.command {
        Some(Command::Dev) => Mode::Dev,
        _ => Mode::Build
    };

    Config::new(&*cli.outdir,
                &*cli.indir,
                &*cli.templatedir,
                &*cli.prefix,
                mode )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // install global subscriber configured based on RUST_LOG envvar.
    tracing_subscriber::fmt::init();
    info!("Logging enabled");

    let cli = Cli::parse();
    match cli.command {
        None => println!("\nuse command 'dev' for  watch server or 'build' for generating static files\n\n"),
        Some(ref cmd) => {
            let config:Config = cli_config(&cli);
            match cmd {
                Command::Dev => watch::run(&config).await?,
                Command::Build => {
                    let _hbs = setup::init(&config)?;
                }
            }
        }
    };

    Ok(())
}


