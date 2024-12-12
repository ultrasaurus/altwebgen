use anyhow::anyhow;
use clap::Parser;
use std::{
    fs,
    io::BufReader,
    path::PathBuf
};
use words::{html_words, HtmlWords, WordTime};

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

    let HtmlWords{html:html_string, word_index:_, last_timing_index:_} = match cli.input {
        None => html_words(&cli.text, None)?,
        Some(path_string) => {
            println!("Text input path: {}", path_string);
            let txt_path = PathBuf::from(path_string);
            let transcript_path = txt_path.with_extension("transcript.json");
            let text: String = fs::read_to_string(txt_path)?;
            println!("Checking for transcript file: {}", transcript_path.display());
            match fs::File::open(transcript_path) {
                Ok(file) => {
                    let transcript_reader =
                        BufReader::new(file);
                    let timing =
                        WordTime::from_transcript(transcript_reader)?;
                    html_words(&text, Some(&timing))?
                },
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        println!("No transcript file found: rendering HTML without word timing");
                        html_words(&text, None)?
                    } else {
                        // Err(anyhow!(e)
                        // .context("transcript file could not be opened {}", txt_path))
                        return Err(anyhow!(e).context("from_transcript: failed to convert to json"))
                    }
                }
            }
        }
    };

    match cli.output {
        None =>   println!("{}", html_string),
        Some(path) => std::fs::write(path, html_string)?
    }
  

    Ok(())
}
