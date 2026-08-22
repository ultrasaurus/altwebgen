//-- Path utlity functions -----------------------------------------------
// extensons to Path struct and related helper functions
use mime::Mime;
use new_mime_guess as mime_guess;
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};
use tracing::trace;

#[allow(dead_code)]
pub trait PathExt {
    // given a path, ensure that all parent directories of that path exist
    // and create any that don't exist
    fn create_all_parent_dir(&self) -> std::io::Result<()>;
    fn get_ext(&self) -> Option<Cow<'static, str>>;
    fn get_ext_str(&self) -> Option<&str>;
    fn mimetype(&self) -> Option<Mime>;
    fn is_markdown(&self) -> bool;
    // strips up to two extensions, e.g. "foo.md.hbs" -> "foo",
    // "foo.transcript.json" -> "foo", "foo.md" -> "foo". used to identify a
    // document by its "stem" regardless of which of these extension
    // combinations it was written with.
    fn double_stem(&self) -> PathBuf;
}

impl PathExt for Path {
    fn create_all_parent_dir(&self) -> std::io::Result<()> {
        let dir = self.parent().unwrap();
        if !dir.exists() {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }

    fn get_ext_str(&self) -> Option<&str> {
        if let Some(ext_osstr) = self.extension() {
            ext_osstr.to_str()
        } else {
            None
        }
    }
    fn get_ext(&self) -> Option<Cow<'static, str>> {
        self.extension()
            .map(|ext_osstr| Cow::Owned(ext_osstr.to_string_lossy().to_lowercase()))
    }
    fn mimetype(&self) -> Option<Mime> {
        trace!("PathExt mimetype {}", self.display());
        let result = mime_guess::from_path(self).first();
        if let Some(found) = result {
            if found.type_() == mime::AUDIO && found.subtype() == "m4a" {
                Some("audio/mp4".parse::<Mime>().unwrap())
            } else {
                Some(found)
            }
        } else {
            None
        }
    }
    fn is_markdown(&self) -> bool {
        if let Some(ext) = self.extension() {
            if ext == "md" || ext == "markdown" {
                return true;
            }
        }
        false
    }
    fn double_stem(&self) -> PathBuf {
        self.with_extension("").with_extension("")
    }
}

#[cfg(test)]
mod tests {
    // importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_get_ext_png() {
        let result = Path::new("foo.png").get_ext().unwrap();
        assert_eq!(result, "png".to_string());
    }
    #[test]
    fn test_get_ext_empty() {
        let result = Path::new("").get_ext();
        assert_eq!(result, None);
    }
    #[test]
    fn test_double_stem_two_extensions() {
        assert_eq!(Path::new("foo.md.hbs").double_stem(), PathBuf::from("foo"));
    }
    #[test]
    fn test_double_stem_one_extension() {
        assert_eq!(Path::new("foo.md").double_stem(), PathBuf::from("foo"));
    }
    #[test]
    fn test_double_stem_no_extension() {
        assert_eq!(Path::new("foo").double_stem(), PathBuf::from("foo"));
    }
    #[test]
    fn test_imetype_png() {
        let result = Path::new("foo.png").mimetype();
        assert_eq!(result, Some(mime::IMAGE_PNG));
    }
    #[test]
    fn test_get_mimetype_empty() {
        let result = Path::new("foo").mimetype();
        assert_eq!(result, None);
    }
}
