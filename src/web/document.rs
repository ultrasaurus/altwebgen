use mime::Mime;
use std::path::{Path, PathBuf};
use crate::Config;
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
    pub fn outpath(&self, config: &Config) -> anyhow::Result<PathBuf> {
        let stem = config.outpath(&self.path)?;
        let path = match self.mime.subtype().as_str() {
            "x-handlebars-template" => stem.with_extension(""),
            _ => stem.with_extension("html")

        };
        Ok(path)
    }

}
