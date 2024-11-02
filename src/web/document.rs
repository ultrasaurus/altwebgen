use mime::Mime;
use std::path::{Path, PathBuf};
use crate::util::PathExt;

pub struct Document {
    pub path: PathBuf,
    pub mime: Mime
}

impl Document {
    pub fn from_path<P: AsRef<Path>>(document_path: P) -> Self {
        let path = document_path.as_ref();
        Document {
            path: PathBuf::from(path),
            mime: {
                match path.mimetype() {
                    Some(mimetype) => mimetype,
                    None => mime::APPLICATION_OCTET_STREAM
                }
            }
        }
    }
}
