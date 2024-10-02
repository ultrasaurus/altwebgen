use anyhow::{anyhow, bail};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf}
};
use tracing::{info, error, trace};
use walkdir::WalkDir;

mod dir_entry;
pub use self::dir_entry::DirEntryExt;

pub mod path;
pub use self::path::PathExt;

use crate::config::Config;

pub fn read_file_to_string<P: AsRef<Path>>(filepath: P) -> anyhow::Result<String> {
    let path = std::fs::canonicalize(filepath)?;
    trace!("read_file #{}", path.display());
    let mut f = File::open(path)?;
    let mut buf = String::new();
    let bytes = f.read_to_string(&mut buf)?;
    if bytes == 0 {
        bail!("failed to read: 0 bytes returned from read_to_string");
    }
    Ok(buf)
}


pub fn get_current_working_dir() -> std::io::Result<PathBuf> {
    let wd = std::env::current_dir()?;
    info!("working directory: {}", wd.display());
    Ok(wd)
}

// copy a directory with all of its files recursively
pub fn copy_dir_all<P: AsRef<Path>>(src: P, dst: &Path) -> anyhow::Result<()> {
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


pub fn create_destdir(config: &Config, sourcepath: &Path) -> anyhow::Result<()> {
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

pub fn clean_and_recreate_dir<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
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
