use anyhow::{anyhow, bail};
use std::{
    collections::HashSet,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use tracing::{error, info, trace, warn};
use walkdir::WalkDir;

mod dir_entry;
pub use self::dir_entry::DirEntryExt;

mod node_ref;
#[allow(unused)]
pub use node_ref::NodeRefExt;

pub mod path;
pub use self::path::PathExt;

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
    let dst_path: &Path = dst;
    let dst_dir = dst_path.to_path_buf();
    info!(
        "copy_dir_all from: {} to: {}",
        src.as_ref().display(),
        dst_path.display()
    );

    for entry_result in WalkDir::new(&src) {
        let entry = entry_result.map_err(|_| {
            anyhow!(
                "invalid DirEntry, failed to copy directory, from {} to {}",
                src.as_ref().display(),
                dst_path.display()
            )
        })?;

        let from = entry.path();
        let to = dst_dir.join(from.strip_prefix(&src)?);
        trace!("copy {} => {}", from.display(), to.display());

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
            std::fs::copy(from, &to).map_err(|_| {
                anyhow!(
                    "copy_dir_all: failed to copy file, from {} to {}",
                    from.display(),
                    to.display()
                )
            })?;
        }
        // ignore the rest
        else {
            warn!("copy: ignored symlink {}", from.display());
        }
    }
    Ok(())
}

pub fn create_destdir(sourcedir: &Path, outdir: &Path, sourcepath: &Path) -> anyhow::Result<()> {
    let rel_path = sourcepath.strip_prefix(sourcedir);
    if rel_path.is_err() {
        let err_report = format!(
            "expected strip prefix match for soucepath {} and sourcedir {}",
            sourcepath.display(),
            sourcedir.display()
        );
        error!(err_report);
        anyhow::bail!(err_report);
    } else {
        let dest_path = outdir.join(rel_path?);
        let result = std::fs::create_dir_all(&dest_path);
        if result.is_err() {
            anyhow::bail!("failed to create {}", &dest_path.display())
        }
    }
    Ok(())
}

// creates the output directory (and its media directory) if missing,
// leaving what's already there so unchanged files don't need rewriting
pub fn ensure_outdir(path: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(path.join("media")).map_err(|e| {
        anyhow!(
            "failed to create directory: {}, error: {}",
            path.display(),
            e
        )
    })
}

// writes `bytes` to `path` unless the file already holds exactly that
pub fn write_if_changed(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    if let Ok(existing) = std::fs::read(path) {
        if existing == bytes {
            return Ok(());
        }
    }
    std::fs::write(path, bytes)
        .map_err(|e| anyhow!("failed to write {}, error: {}", path.display(), e))
}

// copies `from` to `to` unless `to` is the same size and no older, so
// large media files aren't read or copied on every build
pub fn copy_if_changed(from: &Path, to: &Path) -> anyhow::Result<()> {
    if let (Ok(src), Ok(dest)) = (std::fs::metadata(from), std::fs::metadata(to)) {
        if let (Ok(src_time), Ok(dest_time)) = (src.modified(), dest.modified()) {
            if src.len() == dest.len() && dest_time >= src_time {
                return Ok(());
            }
        }
    }
    std::fs::copy(from, to).map_err(|e| {
        anyhow!(
            "failed to copy {} to {}, error: {}",
            from.display(),
            to.display(),
            e
        )
    })?;
    Ok(())
}

// removes files under `dir` that aren't in `keep`, then any directories
// left empty that aren't in `keep` either. returns how many were removed.
pub fn remove_stale(dir: &Path, keep: &HashSet<PathBuf>) -> anyhow::Result<usize> {
    let mut removed = 0;
    for entry in WalkDir::new(dir).min_depth(1).contents_first(true) {
        let entry = entry?;
        let path = entry.path();
        if keep.contains(path) {
            continue;
        }
        if entry.file_type().is_dir() {
            if std::fs::read_dir(path)?.next().is_none() {
                std::fs::remove_dir(path)?;
                removed += 1;
            }
        } else {
            trace!("remove stale output: {}", path.display());
            std::fs::remove_file(path)?;
            removed += 1;
        }
    }
    Ok(removed)
}

pub fn clean_and_recreate_dir<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
    let path_ref = path.as_ref();
    if path_ref.exists() {
        std::fs::remove_dir_all(path_ref).map_err(|e| {
            anyhow!(format!(
                "failed to delete directory: {}, error: {}",
                path_ref.display(),
                e
            ))
        })?;
    }
    std::fs::create_dir_all(path_ref).map_err(|e| {
        anyhow!(format!(
            "failed to create directory: {}, error: {}",
            path_ref.display(),
            e
        ))
    })?;
    let media_dir = path_ref.to_path_buf().join("media");
    std::fs::create_dir(&media_dir)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir(name: &str) -> PathBuf {
        let dir = PathBuf::from("target/test-out/util").join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn modified(path: &Path) -> std::time::SystemTime {
        std::fs::metadata(path).unwrap().modified().unwrap()
    }

    #[test]
    fn write_if_changed_leaves_same_content_alone() {
        let dir = test_dir("write_if_changed");
        let path = dir.join("a.html");
        write_if_changed(&path, b"one").unwrap();
        let first = modified(&path);
        std::thread::sleep(std::time::Duration::from_millis(20));
        write_if_changed(&path, b"one").unwrap();
        assert_eq!(modified(&path), first);
        write_if_changed(&path, b"two").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"two");
    }

    #[test]
    fn copy_if_changed_copies_new_and_edited_files() {
        let dir = test_dir("copy_if_changed");
        let (from, to) = (dir.join("from.txt"), dir.join("to.txt"));
        std::fs::write(&from, "one").unwrap();
        copy_if_changed(&from, &to).unwrap();
        assert_eq!(std::fs::read(&to).unwrap(), b"one");
        let copied = modified(&to);
        std::thread::sleep(std::time::Duration::from_millis(20));
        copy_if_changed(&from, &to).unwrap();
        assert_eq!(modified(&to), copied, "unchanged source isn't copied again");
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(&from, "two").unwrap();
        copy_if_changed(&from, &to).unwrap();
        assert_eq!(std::fs::read(&to).unwrap(), b"two");
    }

    #[test]
    fn remove_stale_keeps_listed_paths() {
        let dir = test_dir("remove_stale");
        for p in ["keep.html", "old.html", "post/index.html", "gone/old.html"] {
            let path = dir.join(p);
            path.create_all_parent_dir().unwrap();
            std::fs::write(path, "x").unwrap();
        }
        std::fs::create_dir_all(dir.join("empty-but-kept")).unwrap();
        let keep: HashSet<PathBuf> = ["keep.html", "post/index.html", "empty-but-kept"]
            .iter()
            .map(|p| dir.join(p))
            .collect();
        assert_eq!(remove_stale(&dir, &keep).unwrap(), 3);
        assert!(dir.join("keep.html").exists());
        assert!(dir.join("post/index.html").exists());
        assert!(dir.join("empty-but-kept").exists());
        assert!(!dir.join("old.html").exists());
        assert!(!dir.join("gone").exists());
    }
}
