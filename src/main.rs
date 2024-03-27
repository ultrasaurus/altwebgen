use clap::Parser;
use words::html_words;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// source text 
    #[clap(short, long, value_parser, default_value = "Hello world!")]
    text: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let html_string = html_words(&cli.text)?;

    println!("-----");
    println!("{}", html_string);

    println!("-----");

    Ok(())
}
