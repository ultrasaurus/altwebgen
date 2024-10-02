use crate::util::*;
use mime::Mime;
use pulldown_cmark::CowStr;
use std::path::Path;

pub trait CowStrExt {
    fn mimetype(&self) -> Option<Mime>;
}

impl CowStrExt for CowStr<'_> {
    fn mimetype(&self) -> Option<Mime> {
        // prolly not efficient for some cases, consider later refactor
        let url = self.to_string();
        let path = Path::new(&url);
        path.mimetype()
    }
}
