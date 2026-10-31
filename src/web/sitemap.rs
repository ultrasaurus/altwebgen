use crate::web::date::FmDate;
use crate::web::md::Attrs;
use serde_json::Value;
use std::fmt::Write;
use std::path::Path;

// sitemap.xml for search engines, written when `_site.yaml` has
// `site_url`, since a sitemap needs absolute URLs
#[derive(Debug, Default)]
pub struct Sitemap {
    // (path from the site root, lastmod)
    entries: Vec<(String, Option<String>)>,
}

// the site's address from `_site.yaml`, without a trailing slash
pub fn site_url(site_attr: &Attrs) -> Option<String> {
    let url = site_attr.get("site_url")?.as_str()?.trim_end_matches('/');
    (!url.is_empty()).then(|| url.to_string())
}

// "a/b/index.html" -> "a/b/"; "about.html" -> "about.html"
fn served_path(rel: &Path) -> String {
    let parts: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();
    let path = parts.join("/");
    match path.strip_suffix("index.html") {
        Some(dir) if dir.is_empty() || dir.ends_with('/') => dir.to_string(),
        _ => path,
    }
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

impl Sitemap {
    // a page written to `rel` (relative to the output directory), unless
    // its frontmatter has `sitemap: false` or it's a draft (`dev --drafts`);
    // lastmod is `updated`, or `date`
    pub fn add(&mut self, rel: &Path, attrs: Option<&Attrs>) {
        let flag = |key: &str| attrs.and_then(|a| a.get(key)).and_then(Value::as_bool);
        if flag("sitemap") == Some(false) || flag("draft") == Some(true) {
            return;
        }
        let lastmod = attrs.and_then(|a| {
            ["updated", "date"]
                .iter()
                .find_map(|key| a.get(*key).and_then(FmDate::from_value))
                .map(|date| date.w3c_date())
        });
        self.entries.push((served_path(rel), lastmod));
    }

    // the XML, with each path after `site_url` and the path prefix
    pub fn xml(&self, site_url: &str, prefix: &str) -> String {
        let mut entries: Vec<&(String, Option<String>)> = self.entries.iter().collect();
        entries.sort();
        let mut xml = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
        );
        for (path, lastmod) in entries {
            let loc = escape(&format!("{site_url}{prefix}{path}"));
            let _ = write!(xml, "  <url><loc>{loc}</loc>");
            if let Some(lastmod) = lastmod {
                let _ = write!(xml, "<lastmod>{lastmod}</lastmod>");
            }
            xml.push_str("</url>\n");
        }
        xml.push_str("</urlset>\n");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::md;

    #[test]
    fn site_url_without_trailing_slash() {
        let attrs = md::parse_attrs("site_url: https://example.com/").unwrap();
        assert_eq!(site_url(&attrs).as_deref(), Some("https://example.com"));
        assert_eq!(site_url(&Attrs::new()), None);
        assert_eq!(site_url(&md::parse_attrs("site_url: ''").unwrap()), None);
    }

    #[test]
    fn served_path_drops_index_html() {
        assert_eq!(served_path(Path::new("index.html")), "");
        assert_eq!(
            served_path(Path::new("2009/04/slug/index.html")),
            "2009/04/slug/"
        );
        assert_eq!(served_path(Path::new("about.html")), "about.html");
        assert_eq!(served_path(Path::new("not-index.html")), "not-index.html");
    }

    #[test]
    fn xml_sorted_with_lastmod_and_opt_out() {
        let mut sitemap = Sitemap::default();
        let post = md::parse_attrs("date: '2009-04-17T10:00:00-07:00'").unwrap();
        let updated = md::parse_attrs("date: 2009-04-01\nupdated: 2010-05").unwrap();
        let hidden = md::parse_attrs("sitemap: false").unwrap();
        let draft = md::parse_attrs("draft: true").unwrap();
        sitemap.add(Path::new("2009/04/b&c/index.html"), Some(&post));
        sitemap.add(Path::new("about.html"), Some(&updated));
        sitemap.add(Path::new("404.html"), Some(&hidden));
        sitemap.add(Path::new("drafts.html"), Some(&draft));
        sitemap.add(Path::new("index.html"), None);
        assert_eq!(
            sitemap.xml("https://example.com", "/"),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n  \
             <url><loc>https://example.com/</loc></url>\n  \
             <url><loc>https://example.com/2009/04/b&amp;c/</loc><lastmod>2009-04-17</lastmod></url>\n  \
             <url><loc>https://example.com/about.html</loc><lastmod>2010-05</lastmod></url>\n\
             </urlset>\n"
        );
    }
}
