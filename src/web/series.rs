use crate::util::PathExt;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use tracing::warn;

// describes an ordered series of pages (e.g. a multi-page booklet), read from
// a `series.yaml` file that sits alongside the pages it describes. any page
// in the same directory automatically picks up prev/next/toc navigation vars
// if its slug (filename with extensions stripped) appears in `pages`.
#[derive(Debug, Clone, Deserialize)]
pub struct Series {
    pub title: Option<String>,
    // slug of the table-of-contents page for this series
    pub toc: String,
    // ordered slugs of the pages in this series (not including toc)
    pub pages: Vec<String>,
}

impl Series {
    // looks for series.yaml next to `path`; returns None if there isn't one
    // or it fails to parse (logging a warning in the latter case, since a
    // malformed series.yaml is a content error worth surfacing rather than
    // silently leaving pages without navigation).
    pub fn load_sibling(path: &Path) -> Option<Series> {
        let series_path = path.parent()?.join("series.yaml");
        if !series_path.exists() {
            return None;
        }
        let f = std::fs::File::open(&series_path).ok()?;
        match serde_yaml::from_reader(f) {
            Ok(series) => Some(series),
            Err(e) => {
                warn!(
                    "series.yaml at {} failed to parse: {}",
                    series_path.display(),
                    e
                );
                None
            }
        }
    }

    // prev/next/toc/series-title/series-position template vars for `slug`.
    // returns an empty map if `slug` isn't the toc page and isn't one of
    // this series' pages either.
    pub fn nav_vars(&self, slug: &str) -> HashMap<String, String> {
        let mut vars = HashMap::new();

        // the toc page itself: no prev (nothing comes before it), no
        // position (it isn't numbered among the pages), and no self-link
        // back to the toc - just a title and a forward arrow into page one.
        if slug == self.toc {
            if let Some(title) = &self.title {
                vars.insert("series-title".into(), title.clone());
            }
            if let Some(first) = self.pages.first() {
                vars.insert("next".into(), format!("{}.html", first));
            }
            return vars;
        }

        let Some(pos) = self.pages.iter().position(|p| p == slug) else {
            return vars;
        };

        vars.insert("toc".into(), format!("{}.html", self.toc));
        if let Some(title) = &self.title {
            vars.insert("series-title".into(), title.clone());
        }
        vars.insert(
            "series-position".into(),
            format!("{} of {}", pos + 1, self.pages.len()),
        );

        let prev = if pos == 0 {
            self.toc.clone()
        } else {
            self.pages[pos - 1].clone()
        };
        vars.insert("prev".into(), format!("{}.html", prev));

        if let Some(next) = self.pages.get(pos + 1) {
            vars.insert("next".into(), format!("{}.html", next));
        }

        vars
    }
}

// identifies a page for series lookup purposes: its filename with up to two
// extensions stripped (matching how ref files are stemmed), e.g.
// "intro.md.hbs" -> "intro".
pub fn page_slug(path: &Path) -> String {
    path.double_stem()
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_series() -> Series {
        Series {
            title: Some("Sample Booklet".to_string()),
            toc: "index".to_string(),
            pages: vec!["intro".into(), "middle".into(), "end".into()],
        }
    }

    #[test]
    fn nav_vars_first_page_prev_is_toc() {
        let vars = sample_series().nav_vars("intro");
        assert_eq!(vars.get("prev"), Some(&"index.html".to_string()));
        assert_eq!(vars.get("next"), Some(&"middle.html".to_string()));
        assert_eq!(vars.get("toc"), Some(&"index.html".to_string()));
        assert_eq!(
            vars.get("series-title"),
            Some(&"Sample Booklet".to_string())
        );
        assert_eq!(vars.get("series-position"), Some(&"1 of 3".to_string()));
    }

    #[test]
    fn nav_vars_toc_page_gets_title_and_next_only() {
        let vars = sample_series().nav_vars("index");
        assert_eq!(
            vars.get("series-title"),
            Some(&"Sample Booklet".to_string())
        );
        assert_eq!(vars.get("next"), Some(&"intro.html".to_string()));
        assert_eq!(vars.get("prev"), None);
        assert_eq!(vars.get("toc"), None);
        assert_eq!(vars.get("series-position"), None);
    }

    #[test]
    fn nav_vars_middle_page() {
        let vars = sample_series().nav_vars("middle");
        assert_eq!(vars.get("prev"), Some(&"intro.html".to_string()));
        assert_eq!(vars.get("next"), Some(&"end.html".to_string()));
        assert_eq!(vars.get("series-position"), Some(&"2 of 3".to_string()));
    }

    #[test]
    fn nav_vars_last_page_has_no_next() {
        let vars = sample_series().nav_vars("end");
        assert_eq!(vars.get("prev"), Some(&"middle.html".to_string()));
        assert_eq!(vars.get("next"), None);
        assert_eq!(vars.get("series-position"), Some(&"3 of 3".to_string()));
    }

    #[test]
    fn nav_vars_unlisted_slug_returns_empty() {
        let vars = sample_series().nav_vars("not-in-series");
        assert!(vars.is_empty());
    }

    #[test]
    fn page_slug_strips_double_extension() {
        assert_eq!(page_slug(Path::new("dir/intro.md.hbs")), "intro");
        assert_eq!(page_slug(Path::new("dir/index.md")), "index");
    }

    #[test]
    fn load_sibling_returns_none_when_no_series_file() {
        assert!(Series::load_sibling(Path::new("src/test/data/short-sentence.md")).is_none());
    }
}
