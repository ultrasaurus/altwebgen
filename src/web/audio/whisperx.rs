use std::{fs, path::Path};
use std::process::Command;
use tracing::{error, info, trace, warn};

pub fn gen_transcript(
        audio: impl AsRef<Path>,
        transcript: impl AsRef<Path>
) -> anyhow::Result<()> {
    let root_path = Path::new(".");
    let outpath = transcript.as_ref().parent().unwrap_or(root_path);

    generate_whisperx_json(audio, outpath)
    // TODO: convert transcript to podcast format
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
        let outfile = Path::new("src/test/data");
        generate_whisperx_json(&infile, &outfile).unwrap();
    }

    #[test]
    fn test_gen_transcript() {
        let infile = Path::new("src/test/data/short-sentence.mp3");
        let outfile = Path::new("src/test/data/short-sentence.transcript.json");
        generate_whisperx_json(&infile, &outfile).unwrap();
    }
}


