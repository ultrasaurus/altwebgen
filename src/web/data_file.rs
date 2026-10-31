use crate::util::read_file_to_string;
use crate::web::md::Attrs;
use crate::web::series::page_slug;
use std::path::{Path, PathBuf};
use tracing::warn;

// data files: `<slug>.<name>.yaml` next to a page `<slug>.md` (or
// `.md.hbs`, `.html.hbs`) gives that page the template variable `<name>`,
// holding the file's whole YAML value. a data file isn't published; a
// `.yaml` file with no page of that slug next to it is copied as usual

const PAGE_EXTENSIONS: [&str; 3] = [".md", ".md.hbs", ".html.hbs"];

// (slug, name) if `path` is named like a data file
fn split_name(path: &Path) -> Option<(&str, &str)> {
    let file = path.file_name()?.to_str()?;
    let (slug, name) = file.strip_suffix(".yaml")?.rsplit_once('.')?;
    (!slug.is_empty() && !name.is_empty()).then_some((slug, name))
}

fn has_page(dir: &Path, slug: &str) -> bool {
    PAGE_EXTENSIONS
        .iter()
        .any(|ext| dir.join(format!("{slug}{ext}")).is_file())
}

// true for a data file, which belongs to a page and isn't published
pub fn is_data_file(path: &Path) -> bool {
    match (split_name(path), path.parent()) {
        (Some((slug, _)), Some(dir)) => has_page(dir, slug),
        _ => false,
    }
}

// the values of the data files for the page at `path`, by variable name; a
// file that isn't valid YAML is reported and ignored
pub fn page_data(path: &Path) -> Attrs {
    let mut data = Attrs::new();
    let slug = page_slug(path);
    let Some(entries) = path.parent().and_then(|dir| std::fs::read_dir(dir).ok()) else {
        return data;
    };
    let mut files: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    files.sort();
    for file in files {
        let Some((file_slug, name)) = split_name(&file) else {
            continue;
        };
        if file_slug != slug {
            continue;
        }
        let value = read_file_to_string(&file).and_then(|yaml| {
            let value: serde_yaml::Value = serde_yaml::from_str(&yaml)?;
            Ok(serde_json::to_value(value)?)
        });
        match value {
            Ok(value) => {
                data.insert(name.to_string(), value);
            }
            Err(e) => warn!("'{}': isn't valid YAML, ignored: {e}", file.display()),
        }
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    // a fresh temp folder holding `files` (name, contents)
    fn folder(name: &str, files: &[(&str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("altwebgen-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for (file, contents) in files {
            std::fs::write(dir.join(file), contents).unwrap();
        }
        dir
    }

    #[test]
    fn split_name_needs_slug_and_name() {
        assert_eq!(
            split_name(Path::new("a/hello.reviews.yaml")),
            Some(("hello", "reviews"))
        );
        assert_eq!(split_name(Path::new("series.yaml")), None);
        assert_eq!(split_name(Path::new(".x.yaml")), None);
        assert_eq!(split_name(Path::new("hello.reviews.yml")), None);
    }

    #[test]
    fn page_data_lists_and_mappings() {
        let dir = folder(
            "data-files",
            &[
                ("hello.md", "Hello"),
                ("hello.reviews.yaml", "- good\n- great\n"),
                ("hello.stats.yaml", "views: 3\n"),
                ("other.notes.yaml", "- not for hello\n"),
            ],
        );
        let data = page_data(&dir.join("hello.md"));
        assert_eq!(data.len(), 2);
        assert_eq!(data["reviews"][1], "great");
        assert_eq!(data["stats"]["views"], 3);
    }

    #[test]
    fn page_data_bad_yaml_is_ignored() {
        let dir = folder(
            "bad-data-file",
            &[("hello.md", "Hello"), ("hello.broken.yaml", "a: [")],
        );
        assert!(page_data(&dir.join("hello.md")).is_empty());
    }

    #[test]
    fn yaml_without_a_page_is_published() {
        let dir = folder(
            "data-file-page",
            &[
                ("hello.md.hbs", "Hello"),
                ("hello.reviews.yaml", "[]"),
                ("feed.v2.yaml", "[]"),
            ],
        );
        assert!(is_data_file(&dir.join("hello.reviews.yaml")));
        assert!(!is_data_file(&dir.join("feed.v2.yaml")));
    }
}
