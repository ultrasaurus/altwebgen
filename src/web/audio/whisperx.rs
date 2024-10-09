use std::path::Path;
use std::process::Command;
use tracing::warn;
use std::fs::OpenOptions;

pub fn gen_transcript<P: AsRef<Path>>(audio: P, transcript: P) -> anyhow::Result<()> {
    generate_whisperx_json(audio, transcript)
}

fn generate_whisperx_json<P: AsRef<Path>>(audio: P, transcript: P) -> anyhow::Result<()> {
    let outpath = transcript.as_ref();
    if outpath.is_file() {
        warn!("existing whisper transcript file will be overwritten: {}", outpath.to_string_lossy())
    }
    let f = OpenOptions::new()
        .write(true).create(true)
        .open(outpath)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_gen_transcript() {
        let infile = Path::new("src/test/data/short-sentence.mp3");
        let outfile = Path::new("src/test/data/short-sentence..whisperx.json");
        // let outfile = infile.with_extension(".whisperx.json");
        gen_transcript(&infile, &outfile).unwrap();
    }
}



#[cfg(tests)]
mod tests {
    use super::*;

    #[test]
    fn test_whisperx() {
    }
}