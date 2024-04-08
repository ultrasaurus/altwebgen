use anyhow::anyhow;
use handlebars::Handlebars;
use new_mime_guess as mime_guess;
use mime::Mime;
use std::{default::Default, path::{Path, PathBuf}};
use std::io::Write;

use tracing::{info, error, trace};
use walkdir::WalkDir;
use warp::ws::Message;

mod web;
mod util;
mod config;
use config::Config;
mod devserve;
use util::*;

mod watch;
use watch::watch;

fn get_current_working_dir() -> std::io::Result<PathBuf> {
    let wd = std::env::current_dir()?;
    info!("working directory: {}", wd.display());
    Ok(wd)
}

// copy a directory with all of its files recursively
fn copy_dir_all<P: AsRef<Path>>(src: P, dst: &Path) -> anyhow::Result<()> {
    let dst_path: &Path = dst.as_ref();
    let dst_dir = dst_path.to_path_buf();
    for entry_result in WalkDir::new(&src) {
        let entry = entry_result.map_err(|_| {
            anyhow!("invalid DirEntry, failed to copy directory, from {} to {}",
                src.as_ref().display(),
                dst_path.display())
        })?;

        let from = entry.path();
        let to = dst_dir.join(from.strip_prefix(&src)?);
        println!("\tcopy {} => {}", from.display(), to.display());

        // create directories
        if entry.file_type().is_dir() {
            if let Err(e) = std::fs::create_dir(to) {
                match e.kind() {
                    std::io::ErrorKind::AlreadyExists => {}
                    _ => return Err(e.into()),
                }
            }
        }
        // copy files
        else if entry.file_type().is_file() {
            std::fs::copy(&from, &to).map_err(|_| {
                anyhow!("copy_dir_all: failed to copy file, from {} to {}",
                    from.display(),
                    to.display())
            })?;
        }
        // ignore the rest
        else {
            eprintln!("copy: ignored symlink {}", from.display());
        }
    }
    Ok(())
}


fn create_destdir(config: &Config, sourcepath: &Path) -> anyhow::Result<()> {
    let rel_path = sourcepath
        .strip_prefix(&config.sourcedir);
    if rel_path.is_err() {
        let err_report = format!("expected strip prefix match for soucepath {} and sourcedir {}",
            sourcepath.display(), config.sourcedir.display());
        error!(err_report);
        anyhow::bail!(err_report);
    } else {
        let dest_path = config.outdir.join(rel_path?);
        let result = std::fs::create_dir_all(&dest_path);
        if result.is_err() {
            anyhow::bail!("failed to create {}", &dest_path.display())
        }
    }
    Ok(())
}

fn clean_and_recreate_dir<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
    let path_ref = path.as_ref();
    if path_ref.exists() {
        std::fs::remove_dir_all(path_ref).map_err(|e| {
            anyhow!(format!("failed to delete directory: {}, error: {}", path_ref.display(), e))
        })?;
    }
    std::fs::create_dir_all(path_ref).map_err(|e| {
            anyhow!(format!("failed to create directory: {}, error: {}", path_ref.display(), e))
        })?;
    let media_dir = path_ref.to_path_buf().join("media");
    std::fs::create_dir(&media_dir)?;
    Ok(())
}


fn process_files(config: &Config, handlebars: &Handlebars) -> anyhow::Result<()> {
    info!("Processing files...");
    let walker = WalkDir::new(&config.sourcedir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            !e.is_hidden()
        });

    for entry_result in walker
    {
        trace!("  entry: {:?}", entry_result);
        let entry = entry_result?;
        let path = entry.path();
        if path.is_dir() {
            create_destdir(config, path)?;
        } else {
            web::render_file(config, &handlebars, path)?;
        }
    }

   Ok(())
}

#[derive(Debug, Clone)]
struct AudioFile {
    pub path: PathBuf,
    pub mime: Mime
}
#[derive(Debug, Clone)]
struct Ref {
    pub md: Option<PathBuf>,
    pub audio: Option<AudioFile>,
}
impl Ref {
    pub fn new() -> Self {
        Ref { md: None, audio: None }
    }
    pub fn write_html<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
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
            let html_body = web::md2html(&md)?;
            writer.write(&html_body)?;
        }
        Ok(())
    }
    pub fn write_to_dest(&self, source_dir: &Path, dest_dir: &Path) -> anyhow::Result<()> {
        info!("write_to_dest Ref: {:?}", self);
        if let Some(audio) = &self.audio {
            let source_path = &audio.path;
            let dest_path = PathBuf::from(format!(".dist/media/{}",
                source_path.file_name().unwrap().to_string_lossy()));
            //let dest_path = dest_dir.join(dest_relpath);
            trace!("copy from {:?} to {:?}", &source_path, &dest_path);
            std::fs::copy(source_path, dest_path)?;
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

}

fn process_ref_markdown<P: AsRef<Path>>(source_dir: P, dest_dir:&Path) -> anyhow::Result<()> {
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
                _ => { }  // ignore other file types
            }
        }
    };
    current_ref.write_to_dest(src_dir_path, &dest_dir)?;

    Ok(())
}

fn setup_templates(config: &Config, hbs: &mut Handlebars) -> anyhow::Result<()> {
    info!("setup_templates");
    clean_and_recreate_dir(&config.builddir)?;
    let buildtemplatedir = config.buildtemplatedir();
    copy_dir_all(&config.templatedir, &buildtemplatedir)?;
    let buildrefdir = buildtemplatedir.join("ref");
    std::fs::create_dir_all(&buildrefdir).map_err(|e| {
        anyhow!(format!("failed to create directory: {}, error: {}", &buildrefdir.display(), e))
    })?;

    process_ref_markdown("ref", &buildtemplatedir.join("ref"))?;

        let buildtemplatedir = config.buildtemplatedir();
    hbs.register_templates_directory(&buildtemplatedir, Default::default())
        .map_err(|_| {
            anyhow!("failed to register template directory: {}", buildtemplatedir.display())
        })?;
    info!("Setup: template directory '{}' registered", &buildtemplatedir.display());

    Ok(())
}

// initial setup, called only once
fn setup() -> anyhow::Result<(Config, Handlebars<'static>)> {
    info!("Setup: start");
    info!("       working directory {}", get_current_working_dir()?.display());
    let config:Config = Default::default();
    let mut hbs = Handlebars::new();
    clean_and_recreate_dir(&config.outdir)?;
    std::fs::create_dir_all(&config.sourcedir).map_err(|e| {
        anyhow!(format!("failed to create directory: {}, error: {}", &config.sourcedir.display(), e))
    })?;
    let refdir = "ref";    // TODO: config?
    std::fs::create_dir_all(refdir).map_err(|e| {
        anyhow!(format!("failed to create directory: {}, error: {}", refdir, e))
    })?;
    std::fs::create_dir_all(&config.templatedir).map_err(|e| {
        anyhow!(format!("failed to create directory: {}, error: {}", &config.templatedir.display(), e))
    })?;



    setup_templates(&config, &mut hbs)?;
    process_files(&config, &hbs)?;
    info!("Setup: complete");
    Ok((config, hbs))
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // install global subscriber configured based on RUST_LOG envvar.
    tracing_subscriber::fmt::init();
    info!("Logging enabled");

    let (config, mut hbs) = setup()?;
    let mut source_watch = Vec::new();
    source_watch.push(config.sourcedir.clone());
    let mut template_watch = Vec::new();
    template_watch.push(config.templatedir.clone());
    template_watch.push(PathBuf::from("ref"));

    // A channel used to broadcast to any websockets to reload when a file changes.
    let (tx, _rx) = tokio::sync::broadcast::channel::<Message>(100);
    loop {
        tokio::select! {
            _ = devserve::run(&config, tx.clone()) => {
                error!("unexpected server end");
                break
            },
            source_result = watch(&source_watch) => {
                info!("source watcher result {:?}", source_result);
                clean_and_recreate_dir(&config.outdir)?;
                if let Err(e) = process_files(&config, &hbs) {
                        error!("process_files failed: {:?}", e);
                        break
                } else {
                    let _ = tx.send(Message::text("reload"));
                }
            },
            template_result = watch(&template_watch) => {
                info!("template watcher result {:?}", template_result);
                hbs.clear_templates();
                clean_and_recreate_dir(&config.outdir)?;
                if let Err(e) = setup_templates(&config, &mut hbs) {
                    error!("setup_templates failed: {:?}", e);
                    break
                };
                if let Err(e) = process_files(&config, &hbs) {
                        error!("process_files failed: {:?}", e);
                        break
                } else {
                    let _ = tx.send(Message::text("reload"));
                }
            }
        }
    }
    Ok(())

}


