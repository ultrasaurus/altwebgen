use new_mime_guess as mime_guess;
use mime::Mime;
use std::{
    io::Write,
    path::{Path, PathBuf}
};
use tracing::{info, trace};
use walkdir::WalkDir;

use crate::web::md;
use crate::web;

#[derive(Debug, Clone)]
struct AudioFile {
    pub path: PathBuf,
    pub mime: Mime
}
#[derive(Debug, Clone)]
pub struct Ref {
    md: Option<PathBuf>,
    audio: Option<AudioFile>,
    transcript: Option<PathBuf>,
}
impl Ref {
    pub fn new() -> Self {
        Ref { md: None, audio: None, transcript: None }
    }
    fn write_html<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        trace!("write_html for ref: {:?}", self);
        if let Some(audio) = &self.audio {
            trace!("write_html audio file_name: {:?}", audio.path.file_name());
            let file_name = audio.path.file_name().unwrap().to_string_lossy();
            trace!("write_html audio file_name: {:?}", file_name);
            let url = format!("/media/{}",file_name);
            let link_tag= format!("<a href=\"{}\" title=\"{}\" class=\"audio\"><span class=\"fa-solid fa-play\">{}</span></a>",
                &url, &file_name, &file_name);
            let audio_tag= format!("<audio controls><source src=\"{}\" type=\"{}\">Your browser does not support the audio element. {}</audio>",
                url, audio.mime, &link_tag);
            writer.write(&audio_tag.as_bytes())?;

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
            let dest_path = PathBuf::from(format!(".dist/media/{}",
                source_path.file_name().unwrap().to_string_lossy()));
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
             let writepath = dest_dir
                .join(relpath)
                .with_extension("html.hbs");
             trace!("     writepath: {:?}", writepath);
            let mut writer = std::fs::File::options()
                .create(true)
                .write(true)
                .open(writepath)?;
            self.write_html(&mut writer)?;
        }
        Ok(())
    }

   pub fn process_markdown<P: AsRef<Path>>(source_dir: P, dest_dir:&Path) -> anyhow::Result<()> {
    let src_dir_path = source_dir.as_ref();
    if !src_dir_path.exists() {
        info!("skipping process_ref_markdown, no ref source directory: '{}'", src_dir_path.display());
        return Ok(())
    }
    trace!("process_ref_markdown from '{}' to '{}'", src_dir_path.display(), dest_dir.display());
    // maybe first create a map of stem => Vec[file types]
    let mut prev_stem = None;
    let mut current_ref = Ref::new();
    for e in WalkDir::new(src_dir_path)
        .sort_by(|a,b| a.file_name().cmp(b.file_name()))
        {
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
                current_ref = Ref::new();
                prev_stem = Some(path_stem);
            }
            match (mime.type_(), mime.subtype()) {
                (mime::TEXT, subtype) => {
                    if subtype == "markdown" {
                        current_ref.md = Some(path.to_path_buf())
                    }
                    // else ignore
                },
                (mime::AUDIO, _) => current_ref.audio =
                    Some(AudioFile {
                    path: path.to_path_buf(),
                    mime}),
                (mime::APPLICATION, mime::JSON) => current_ref.transcript =
                    Some(path.to_path_buf()),
                _ => {
                    info!("\n\nignorning unknown file type...\npath: {}\nmime: {}\n\n", path.display(), mime);
                }  // ignore other file types
            }
        }
    };
    current_ref.write_to_dest(src_dir_path, &dest_dir)?;

    Ok(())
}


}
