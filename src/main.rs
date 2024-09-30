use clap::Parser;
use words::html_words;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// source text 
    #[clap(short, long, value_parser, default_value = "Hello world!")]
    text: String,
    /// input file, if --input option is given, then --text is ignored
    #[clap(short, long, value_parser)]
    input: Option<String>,
    /// output file, if not given output to stdio
    #[clap(short, long, value_parser)]
    output: Option<String>,

}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let html_string = match cli.input {
        None => html_words(&cli.text, None)?,
        Some(path) => {
            let input = std::fs::read_to_string(path)?;
            html_words(input, None)?
        }
    };

    match cli.output {
        None =>   println!("{}", html_string),
        Some(path) => std::fs::write(path, html_string)?
    }
  

    Ok(())
}
