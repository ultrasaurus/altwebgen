use mime::Mime;
use new_mime_guess as mime_guess;
use std::{
    collections::BTreeMap,
    io::Write,
    path::{Path, PathBuf},
};
use tracing::{info, trace, warn};
use walkdir::WalkDir;

use crate::{
    config::{Config, Transcript},
    web::{self, md},
};

#[derive(Debug, Clone, PartialEq)]
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

// stem shared by all files belonging to one ref, e.g.
//   ref/talks/hypertext.md, ref/talks/hypertext.mp3, ref/talks/hypertext.transcript.json
// all share the stem ref/talks/hypertext
//
// strips up to two extensions so that "<stem>.transcript.json" collapses to
// the same stem as "<stem>.md" / "<stem>.mp3".
fn ref_stem(path: &Path) -> PathBuf {
    path.with_extension("").with_extension("")
}

// the files collected for one ref stem, before a Config is attached.
// kept separate from Ref so grouping logic can be tested without a Config.
#[derive(Debug, Clone, Default, PartialEq)]
struct RefFiles {
    md: Option<PathBuf>,
    audio: Option<AudioFile>,
    transcript: Option<PathBuf>,
}

impl RefFiles {
    // classify one file into the appropriate slot, based on its mime type.
    // warns (rather than silently overwriting) if a slot is already filled,
    // and warns on any file type not recognized as md/audio/transcript.
    fn merge(&mut self, path: &Path, mime: Mime) {
        match (mime.type_(), mime.subtype()) {
            (mime::TEXT, subtype) if subtype == "markdown" => {
                if self.md.is_some() {
                    warn!(
                        "ref: multiple markdown files for stem '{}', overwriting with '{}'",
                        ref_stem(path).display(),
                        path.display()
                    );
                }
                self.md = Some(path.to_path_buf());
            }
            (mime::AUDIO, _) => {
                if self.audio.is_some() {
                    warn!(
                        "ref: multiple audio files for stem '{}', overwriting with '{}'",
                        ref_stem(path).display(),
                        path.display()
                    );
                }
                self.audio = Some(AudioFile {
                    path: path.to_path_buf(),
                    mime,
                });
            }
            (mime::APPLICATION, mime::JSON) => {
                if self.transcript.is_some() {
                    warn!(
                        "ref: multiple transcript files for stem '{}', overwriting with '{}'",
                        ref_stem(path).display(),
                        path.display()
                    );
                }
                self.transcript = Some(path.to_path_buf());
            }
            _ => {
                warn!(
                    "ref: ignoring unrecognized file type, path: {}, mime: {}",
                    path.display(),
                    mime
                );
            }
        }
    }

    fn into_ref(self, config: &Config) -> Ref<'_> {
        Ref {
            config,
            md: self.md,
            audio: self.audio,
            transcript: self.transcript,
        }
    }
}

fn audio_tag(file_name: &str, audio_mime: &str, url: &str) -> String {
    let link_tag: String= format!("<a href=\"{}\" title=\"{}\" class=\"audio\"><span class=\"fa-solid fa-play\">{}</span></a>",
        &url, &file_name, &file_name);
    format!("<audio id=\"audio\" controls><source src=\"{}\" type=\"{}\">Your browser does not support the audio element. {}</audio>",
        url, audio_mime, &link_tag)
}

impl<'r> Ref<'r> {
    fn write_html<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        trace!("write_html for ref: {:?}", self);
        writer.write_all("<div class='ref'>\n".as_bytes())?;
        if let Some(audio) = &self.audio {
            writer.write_all("<div id='audiotext'>\n".as_bytes())?;
            trace!("write_html audio file_name: {:?}", audio.path.file_name());
            let file_name: &str = audio.path.file_name().unwrap().try_into()?;
            let url = format!("{}media/{}", self.config.prefix, file_name);
            let audio_html = audio_tag(file_name, audio.mime.as_ref(), &url.to_string());
            writer.write_all(audio_html.as_bytes())?;
        }
        if let Some(md) = &self.md {
            let (attrs, html_body) =
                if self.transcript.is_none() || (self.config.transcript == Transcript::Off) {
                    trace!("md::file2html");
                    md::file2html(md)?
                } else {
                    trace!("md::file2html_with_timing");
                    let transcript_path = self.transcript.clone().unwrap();
                    md::file2html_with_timing(&md, &&transcript_path)?
                };
            if !attrs.is_empty() {
                warn!(
                    "ref: '{}' has frontmatter ({}) that isn't used by ref rendering yet, it was stripped and ignored",
                    md.display(),
                    attrs.keys().cloned().collect::<Vec<_>>().join(", ")
                );
            }
            writer.write_all(&html_body)?;
        }
        if self.audio.is_some() {
            writer.write_all("</div>\n".as_bytes())?;
        }
        writer.write_all("</div>\n".as_bytes())?; // closing div class='ref'

        Ok(())
    }
    pub fn write_to_dest(&mut self, source_dir: &Path, dest_dir: &Path) -> anyhow::Result<()> {
        trace!("write_to_dest Ref: {:?}", self);
        trace!(
            "write_to_dest source_dir: {}, dest_dir: {}",
            source_dir.display(),
            dest_dir.display()
        );
        if let Some(audio) = &self.audio {
            let source_path = &audio.path;
            let outdir = &self.config.outdir;
            let dest_path = outdir.join("media").join(source_path.file_name().unwrap());
            trace!("copy from {:?} to {:?}", &source_path, &dest_path);
            std::fs::copy(source_path, dest_path)?;

            if self.transcript.is_none() && self.config.transcript == Transcript::On {
                info!("write_to_dest: no transcript found, attempting to generate one");
                let transcript_path = source_path.with_extension("transcript.json");
                web::audio::gen_transcript(source_path, &transcript_path)?;
                self.transcript = Some(transcript_path);
            }
            if self.md.is_none() {
                warn!(
                    "ref: audio file '{}' has no matching markdown file, media was copied but no ref partial was written",
                    source_path.display()
                );
            }
        }
        if let Some(md) = &self.md {
            let relpath = md.strip_prefix(source_dir)?;
            trace!("     relpath: {:?}", relpath);
            let writepath = dest_dir.join(relpath).with_extension("html.hbs");
            trace!("     writepath: {:?}", writepath);
            let mut writer = std::fs::File::options()
                .create(true)
                .truncate(true)
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

        // group every file under src_dir_path by its ref stem (directory-aware,
        // so refs in different subdirectories never collide just because they
        // happened to land next to each other in the walk order)
        let mut refs: BTreeMap<PathBuf, RefFiles> = BTreeMap::new();
        for e in WalkDir::new(src_dir_path) {
            let entry = e?;
            let path: &Path = entry.path();
            if std::fs::metadata(path)?.is_file() {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                refs.entry(ref_stem(path)).or_default().merge(path, mime);
            }
        }

        for files in refs.into_values() {
            files
                .into_ref(config)
                .write_to_dest(src_dir_path, dest_dir)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const MP3_MIME_STR: &str = "audio/mpeg";

    #[test]
    fn test_audio_tag() {
        let output = audio_tag("hello.mp3", MP3_MIME_STR, "/audio/hello.mp3");
        assert_eq!(output, "<audio id=\"audio\" controls><source src=\"/audio/hello.mp3\" type=\"audio/mpeg\">Your browser does not support the audio element. <a href=\"/audio/hello.mp3\" title=\"hello.mp3\" class=\"audio\"><span class=\"fa-solid fa-play\">hello.mp3</span></a></audio>")
    }

    #[test]
    fn test_ref_stem_single_extension() {
        assert_eq!(
            ref_stem(Path::new("ref/talks/hypertext.md")),
            PathBuf::from("ref/talks/hypertext")
        );
        assert_eq!(
            ref_stem(Path::new("ref/talks/hypertext.mp3")),
            PathBuf::from("ref/talks/hypertext")
        );
    }

    #[test]
    fn test_ref_stem_double_extension() {
        assert_eq!(
            ref_stem(Path::new("ref/talks/hypertext.transcript.json")),
            PathBuf::from("ref/talks/hypertext")
        );
    }

    #[test]
    fn test_ref_stem_no_extension() {
        assert_eq!(
            ref_stem(Path::new("ref/talks/hypertext")),
            PathBuf::from("ref/talks/hypertext")
        );
    }

    #[test]
    fn test_ref_files_merge_classifies_each_kind() {
        let mut files = RefFiles::default();
        files.merge(
            Path::new("ref/hypertext.md"),
            "text/markdown".parse().unwrap(),
        );
        files.merge(
            Path::new("ref/hypertext.mp3"),
            MP3_MIME_STR.parse().unwrap(),
        );
        files.merge(
            Path::new("ref/hypertext.transcript.json"),
            "application/json".parse().unwrap(),
        );

        assert_eq!(files.md, Some(PathBuf::from("ref/hypertext.md")));
        assert_eq!(
            files.audio.map(|a| a.path),
            Some(PathBuf::from("ref/hypertext.mp3"))
        );
        assert_eq!(
            files.transcript,
            Some(PathBuf::from("ref/hypertext.transcript.json"))
        );
    }

    #[test]
    fn test_ref_files_merge_second_md_overwrites_with_warning() {
        let mut files = RefFiles::default();
        files.merge(
            Path::new("ref/hypertext.md"),
            "text/markdown".parse().unwrap(),
        );
        files.merge(
            Path::new("ref/hypertext.markdown"),
            "text/markdown".parse().unwrap(),
        );
        // last one wins; a warning is logged (not asserted here) rather than
        // the first file silently vanishing with no trace
        assert_eq!(files.md, Some(PathBuf::from("ref/hypertext.markdown")));
    }

    #[test]
    fn test_ref_files_merge_unrecognized_type_ignored() {
        let mut files = RefFiles::default();
        files.merge(
            Path::new("ref/hypertext.pdf"),
            "application/pdf".parse().unwrap(),
        );
        assert_eq!(files, RefFiles::default());
    }

    #[test]
    fn test_process_markdown_groups_by_stem() {
        let out_dir = PathBuf::from("target/test-out/ref_markdown/process_markdown_groups/.dist");
        let _ = std::fs::remove_dir_all(&out_dir);
        std::fs::create_dir_all(out_dir.join("media")).unwrap();
        let config = Config {
            outdir: out_dir,
            ..Config::default()
        };
        let dest_dir = PathBuf::from("target/test-out/ref_markdown/process_markdown_groups/ref");
        std::fs::create_dir_all(&dest_dir).unwrap();

        Ref::process_markdown(&config, "src/test/data", &dest_dir).unwrap();

        // two distinct stems from src/test/data should each produce their own partial
        let short_sentence_html =
            std::fs::read_to_string(dest_dir.join("short-sentence.html.hbs")).unwrap();
        assert!(short_sentence_html.contains("annotations"));
        assert!(short_sentence_html.contains("/media/short-sentence.mp3"));

        let no_punctuation_html =
            std::fs::read_to_string(dest_dir.join("short-sentence-no-punctuation.html.hbs"))
                .unwrap();
        // this stem has no audio file in the fixture, so no audio div/media reference
        assert!(no_punctuation_html.contains("<span word='0'"));
        assert!(!no_punctuation_html.contains("audiotext"));
    }

    #[test]
    fn test_write_md() {
        let reference: Ref<'_> = Ref {
            config: &Config::default(),
            md: Some("src/test/data/short-sentence.md".into()),
            audio: None,
            transcript: None,
        };

        //let write_buf = std::io::BufWriter::new(Vec::new());
        let mut write_buf = Vec::new();
        reference.write_html(&mut write_buf).unwrap();

        let output_string = String::from_utf8(write_buf).unwrap();
        let expected =
            "<div class='ref'>\n<p>it may contain annotations, additions and footnotes</p>\n</div>";
        assert_eq!(output_string.trim(), expected);
    }

    #[test]
    fn test_write_md_strips_frontmatter() {
        let reference: Ref<'_> = Ref {
            config: &Config::default(),
            md: Some("src/test/data/with-frontmatter.md".into()),
            audio: None,
            transcript: None,
        };

        let mut write_buf = Vec::new();
        reference.write_html(&mut write_buf).unwrap();

        let output_string = String::from_utf8(write_buf).unwrap();
        assert!(!output_string.contains("title:"));
        assert!(!output_string.contains("Ada Lovelace"));
        assert!(output_string.contains("it may contain annotations"));
    }
    #[test]
    fn test_write_md_audio() {
        // Create AudioFile
        let path = PathBuf::from("src/test/data/short-sentence.mp3");
        let mime = MP3_MIME_STR.parse::<mime::Mime>().unwrap();
        let audio = Some(AudioFile { path, mime });

        let reference: Ref<'_> = Ref {
            config: &Config::default(),
            md: Some("src/test/data/short-sentence.md".into()),
            audio,
            transcript: None,
        };

        let mut write_buf = Vec::new();
        reference.write_html(&mut write_buf).unwrap();

        let output_string = String::from_utf8(write_buf).unwrap();
        let audio_html: String = audio_tag(
            "short-sentence.mp3",
            MP3_MIME_STR,
            "/media/short-sentence.mp3",
        );
        let expected = format!("<div class='ref'>\n<div id='audiotext'>\n{}<p>it may contain annotations, additions and footnotes</p>\n</div>\n</div>", audio_html);
        assert_eq!(output_string.trim(), expected);
    }

    fn create_ref_full(config: &Config) -> Ref<'_> {
        // Create AudioFile struct
        let path = PathBuf::from("src/test/data/short-sentence.mp3");
        let mime = MP3_MIME_STR.parse::<mime::Mime>().unwrap();
        let audio = Some(AudioFile { path, mime });

        Ref {
            config,
            md: Some("src/test/data/short-sentence-no-punctuation.md".into()),
            audio,
            transcript: Some("src/test/data/short-sentence-no-punctuation.transcript.json".into()),
        }
    }
    const EXPECTED_ANNOTATION: &str = "<p><span word='0' start='0.11' end='0.17' debug_body='It'>it</span> <span word='1' start='0.211' end='0.352' debug_body='may'>may</span> <span word='2' start='0.392' end='0.755' debug_body='contain'>contain</span> <span word='3' start='0.876' end='1.622' debug_body='annotations'>annotations</span> <span word='4' start='2.368' end='2.832' debug_body='additions'>additions</span> <span word='5' start='2.893' end='2.973' debug_body='and'>and</span> <span word='6' start='3.034' end='3.498' debug_body='footnotes'>footnotes</span></p>";
    const EXPECTED_TRANSCRIPT_OFF: &str =
        "<p>it may contain annotations additions and footnotes</p>";
    #[test]
    fn test_write_md_audio_transcript() {
        let config = Config::default();
        let reference = create_ref_full(&config);
        let mut write_buf = Vec::new();
        reference.write_html(&mut write_buf).unwrap();

        let output_string = String::from_utf8(write_buf).unwrap();

        let audio_html: String = audio_tag(
            "short-sentence.mp3",
            MP3_MIME_STR,
            "/media/short-sentence.mp3",
        );
        let expected_words = EXPECTED_ANNOTATION;
        let expected = format!(
            "<div class='ref'>\n<div id='audiotext'>\n{}{}\n</div>\n</div>",
            audio_html, expected_words
        );
        assert_eq!(output_string.trim(), expected);
    }

    #[test]
    fn test_write_md_audio_transcript_off() {
        let config = Config {
            transcript: Transcript::Off,
            ..Config::default()
        };
        let reference = create_ref_full(&config);
        let mut write_buf = Vec::new();
        reference.write_html(&mut write_buf).unwrap();

        let output_string = String::from_utf8(write_buf).unwrap();

        let audio_html: String = audio_tag(
            "short-sentence.mp3",
            MP3_MIME_STR,
            "/media/short-sentence.mp3",
        );
        let expected_words = EXPECTED_TRANSCRIPT_OFF;
        let expected = format!(
            "<div class='ref'>\n<div id='audiotext'>\n{}{}\n</div>\n</div>",
            audio_html, expected_words
        );
        assert_eq!(output_string.trim(), expected);
    }
}
