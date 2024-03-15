// DirEntry utlity functions
use walkdir::DirEntry;

pub trait DirEntryExt {
    // return true if the DirEntry represents a hidden file or directory
    fn is_hidden(&self) -> bool;
}

impl DirEntryExt for DirEntry {
    fn is_hidden(&self) -> bool {
        self.file_name()
            .to_str()
            .map(|s| s.starts_with("."))
            .unwrap_or(false)
    }
}
