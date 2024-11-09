use mime::Mime;
use new_mime_guess as mime_guess;
use std::{
    io::Write,
    path::{Path, PathBuf},
};
use tracing::{info, trace};
use walkdir::WalkDir;

use crate::{config::Config, web, web::md};

#[derive(Debug, Clone)]
struct AudioFile {
    pub path: PathBuf,
    pub mime: Mime,
}
#[derive(Debug, Clone)]
pub struct Ref<'a> {
    config: &'a Config,
    md: Option<PathBuf>,
    audio: Option<AudioFile>,
    transcript: Option<PathBuf>,
}

fn audio_tag(file_name: &str, audio_mime: &str, url:&str) -> String {
    let link_tag: String= format!("<a href=\"{}\" title=\"{}\" class=\"audio\"><span class=\"fa-solid fa-play\">{}</span></a>",
        &url, &file_name, &file_name);
    format!("<audio id=\"audio\" controls><source src=\"{}\" type=\"{}\">Your browser does not support the audio element. {}</audio>",
        url, audio_mime, &link_tag)
}

impl<'r> Ref<'r> {
    pub fn new(config: &'r Config) -> Self {
        Ref {
            config,
            md: None,
            audio: None,
            transcript: None,
        }
    }
    fn write_html<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        trace!("write_html for ref: {:?}", self);
        if let Some(audio) = &self.audio {
            trace!("write_html audio file_name: {:?}", audio.path.file_name());
            let file_name: &str = audio.path.file_name().unwrap().try_into()?;
            let url = format!("{}media/{}", self.config.prefix, file_name);
            let audio_html = audio_tag(file_name.into(), &audio.mime.to_string(), &url.to_string());
            writer.write(&audio_html.as_bytes())?;
        }
        if let Some(md) = &self.md {
            let html_body = match &self.transcript {
                None => md::file2html(&md)?,
                Some(transcript_path) => md::file2html_with_timing(&md, &transcript_path)?,
            };
            writer.write(&html_body)?;
        }
        Ok(())
    }
    pub fn write_to_dest(&mut self, source_dir: &Path, dest_dir: &Path) -> anyhow::Result<()> {
        info!("write_to_dest Ref: {:?}", self);
        if let Some(audio) = &self.audio {
            let source_path = &audio.path;
            let dest_path = PathBuf::from(format!(
                ".dist/media/{}",
                source_path.file_name().unwrap().to_string_lossy()
            ));
            //let dest_path = dest_dir.join(dest_relpath);
            trace!("copy from {:?} to {:?}", &source_path, &dest_path);
            std::fs::copy(source_path, dest_path)?;

            if self.transcript == None {
                info!("write_to_dest: no transcript found, attempting to generate one");
                let transcript_path = source_path.with_extension("transcript.json");
                web::audio::gen_transcript(source_path, &transcript_path)?;
                self.transcript = Some(transcript_path);
            }
        }
        if let Some(md) = &self.md {
            let relpath = md.strip_prefix(source_dir)?;
            trace!("     relpath: {:?}", relpath);
            let writepath = dest_dir.join(relpath).with_extension("html.hbs");
            trace!("     writepath: {:?}", writepath);
            let mut writer = std::fs::File::options()
                .create(true)
                .write(true)
                .open(writepath)?;
            self.write_html(&mut writer)?;
        }
        Ok(())
    }

    pub fn process_markdown<P: AsRef<Path>>(
        config: &Config,
        source_dir: P,
        dest_dir: &Path,
    ) -> anyhow::Result<()> {
        let src_dir_path = source_dir.as_ref();
        if !src_dir_path.exists() {
            info!(
                "skipping process_ref_markdown, no ref source directory: '{}'",
                src_dir_path.display()
            );
            return Ok(());
        }
        trace!(
            "process_ref_markdown from '{}' to '{}'",
            src_dir_path.display(),
            dest_dir.display()
        );
        // maybe first create a map of stem => Vec[file types]
        let mut prev_stem = None;
        let mut current_ref = Ref::new(&config);
        for e in WalkDir::new(src_dir_path).sort_by(|a, b| a.file_name().cmp(b.file_name())) {
            let entry = e?;
            let path: &Path = entry.path();
            if std::fs::metadata(path)?.is_file() {
                let path_stem = path.with_extension("").with_extension("");
                trace!("prev_stem: {:?}", prev_stem);
                trace!("path_stem: {}", path_stem.display());
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                if prev_stem != Some(path_stem.clone()) {
                    if prev_stem.is_some() {
                        current_ref.write_to_dest(src_dir_path, &dest_dir)?;
                    }
                    current_ref = Ref::new(config);
                    prev_stem = Some(path_stem);
                }
                match (mime.type_(), mime.subtype()) {
                    (mime::TEXT, subtype) => {
                        if subtype == "markdown" {
                            current_ref.md = Some(path.to_path_buf())
                        }
                        // else ignore
                    }
                    (mime::AUDIO, _) => {
                        current_ref.audio = Some(AudioFile {
                            path: path.to_path_buf(),
                            mime,
                        })
                    }
                    (mime::APPLICATION, mime::JSON) => {
                        current_ref.transcript = Some(path.to_path_buf())
                    }
                    _ => {
                        info!(
                            "\n\nignorning unknown file type...\npath: {}\nmime: {}\n\n",
                            path.display(),
                            mime
                        );
                    } // ignore other file types
                }
            }
        }
        current_ref.write_to_dest(src_dir_path, &dest_dir)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const MP3_MIME_STR:&str = "audio/mpeg";

    #[test]
    fn test_audio_tag() {
        let output = audio_tag("hello.mp3", MP3_MIME_STR, "/audio/hello.mp3");
        assert_eq!(output, "<audio id=\"audio\" controls><source src=\"/audio/hello.mp3\" type=\"audio/mpeg\">Your browser does not support the audio element. <a href=\"/audio/hello.mp3\" title=\"hello.mp3\" class=\"audio\"><span class=\"fa-solid fa-play\">hello.mp3</span></a></audio>")
    }

    #[test]
    fn test_write_md() {
        let reference: Ref<'_> = Ref {config: &Config::default(),
            md: Some("src/test/data/short-sentence.md".into()),
            audio: None,
            transcript: None
        };

        //let write_buf = std::io::BufWriter::new(Vec::new());
        let mut write_buf = Vec::new();
        reference.write_html(&mut write_buf).unwrap();

        let output_string = String::from_utf8(write_buf).unwrap();
        let expected = "<p>it may contain annotations, additions and footnotes</p>";
        assert_eq!(output_string.trim(), expected);
    }
    #[test]
    fn test_write_md_audio() {
        // Create AudioFile
        let path = PathBuf::from("src/test/data/short-sentence.mp3");
        let mime = MP3_MIME_STR.parse::<mime::Mime>().unwrap();
        let audio = Some(AudioFile { path, mime });

        let reference: Ref<'_> = Ref {config: &Config::default(),
            md: Some("src/test/data/short-sentence.md".into()),
            audio,
            transcript: None
        };

        //let write_buf = std::io::BufWriter::new(Vec::new());
        let mut write_buf = Vec::new();
        reference.write_html(&mut write_buf).unwrap();

        let output_string = String::from_utf8(write_buf).unwrap();
        let expected = "<audio id=\"audio\" controls><source src=\"/media/short-sentence.mp3\" type=\"audio/mpeg\">Your browser does not support the audio element. <a href=\"/media/short-sentence.mp3\" title=\"short-sentence.mp3\" class=\"audio\"><span class=\"fa-solid fa-play\">short-sentence.mp3</span></a></audio><p>it may contain annotations, additions and footnotes</p>";
        assert_eq!(output_string.trim(), expected);
    }
}
