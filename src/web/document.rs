use new_mime_guess as mime_guess;
use mime::Mime;
use std::path::{Path, PathBuf};

pub struct Document {
    pub path: PathBuf,
    pub mime: Mime
}

impl Document {
    pub fn from_path<P: AsRef<Path>>(document_path: P) -> Self {
        let path = document_path.as_ref();
        Document {
            path: PathBuf::from(path),
            mime: mime_guess::from_path(path).first_or_octet_stream()
        }
    }
}
