use anyhow::bail;
use std::{fs, path::Path};
use std::process::Command;
use tracing::{error, info, trace, warn};

pub fn gen_transcript(
        audio: impl AsRef<Path>,
        transcript: impl AsRef<Path>
) -> anyhow::Result<()> {
    info!("gen_transcript in: {} out:{}",
        audio.as_ref().display(), transcript.as_ref().display());
    println!("gen_transcript in: {} out:{}",
        audio.as_ref().display(), transcript.as_ref().display());

    let audio_filename = match audio.as_ref().file_name() {
        None => bail!("audio path must refer to a file"),
        Some(name) => name
    };

    let root_path = Path::new(".");
    let out_dir = transcript.as_ref().parent().unwrap_or(root_path);

    generate_whisperx_json(&audio, out_dir)?;
    let audio_outpath = out_dir.join(audio_filename);
    println!("audio_outpath = {}", audio_outpath.display());
    let whisper_json_path = audio_outpath.with_extension("json");
    convert_to_transcript_json(&whisper_json_path, transcript )
}

fn convert_to_transcript_json(
        whisper: impl AsRef<Path>,
        podcast: impl AsRef<Path>
) -> anyhow::Result<()> {
    println!("convert_to_transcript_json in: {} out:{}",
        whisper.as_ref().to_string_lossy(), podcast.as_ref().to_string_lossy());

    let cmd_path = match std::env::home_dir() {
        None => bail!("couldn't find $HOME directory - looking for transcript-converter path"),
        Some(home) => home.join("transcript-converter/transcriptConverter.py")
    };

    // python transcript-converter/transcriptConverter.py
    match Command::new("python")
        .arg(cmd_path.as_os_str())
        .arg(whisper.as_ref().as_os_str())
        .arg(podcast.as_ref().as_os_str())
        .output() {
        Ok(cmd_output) => {
            println!("status: {:?}", cmd_output.status);
            let stdout = String::from_utf8(cmd_output.stdout).unwrap();
            println!("transcriptConverter stdout:\n{}\n\n", stdout);

            let stderr = String::from_utf8(cmd_output.stderr).unwrap();
            println!("transcriptConverter stderr:\n{}\n\n", stderr);
            Ok(())
        },
        Err(e) => {
            Err(anyhow::Error::new(e).context("failed to execute transcriptConverter command"))
        }

    }
}


fn generate_whisperx_json(
    audio: impl AsRef<Path>,
    output_dir: impl AsRef<Path>
) -> anyhow::Result<()> {
    info!("generate_whisperx_json");
    let outpath = output_dir.as_ref();
    let inpath = audio.as_ref();
    fs::create_dir_all(outpath)?;

let output: std::process::Output = Command::new("pwd")
    .output()
    .expect("Failed to execute pwd command");
println!("pwd: {}", String::from_utf8_lossy(&output.stdout));
println!("calling: whisperx {} --language en", inpath.to_string_lossy());

    match Command::new("whisperx")
    .arg(inpath.as_os_str())
    .arg("--output_format")
    .arg("json")
    .arg("--compute_type")
    .arg("float32")
    .arg("--output_dir")
    .arg(outpath.as_os_str())
    .arg("--language")
    .arg("en")
    .output()
    {
        Ok(cmd_output) => {
            println!("status: {:?}", cmd_output.status);
            let stdout = String::from_utf8(cmd_output.stdout).unwrap();
            println!("whisperx stdout:\n{}\n\n", stdout);

            let stderr = String::from_utf8(cmd_output.stderr).unwrap();
            println!("whisperx stderr:\n{}\n\n", stderr);
            Ok(())
        },
        Err(e) => {
            Err(anyhow::Error::new(e).context("failed to execute whisperx command"))
        }
    }
    }

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_generate_whisperx_json() {
        let infile = Path::new("src/test/data/short-sentence.mp3");
        let outdir = Path::new("src/test/data");
        generate_whisperx_json(&infile, &outdir).unwrap();
    }

    #[test]
    fn test_gen_transcript() {
        let infile = Path::new("src/test/data/short-sentence.mp3");
        let outfile = Path::new("src/test/data/short-sentence.transcript.json");
        gen_transcript(&infile, &outfile).unwrap();
    }
}


