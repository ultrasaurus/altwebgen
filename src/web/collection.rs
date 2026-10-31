use crate::config::Config;
use crate::util::*;
use crate::web::date::FmDate;
use crate::web::md::{self, Attrs};
use anyhow::{anyhow, bail};
use globset::{Glob, GlobMatcher};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use walkdir::WalkDir;

// config files that live in the source directory but aren't published
pub const CONFIG_FILES: [&str; 3] = ["collection.yaml", "series.yaml", "_site.yaml"];

// a set of pages that know about each other, e.g. blog posts, described
// by a `collection.yaml` file
#[derive(Debug, Deserialize)]
struct CollectionSpec {
    name: String,
    include: String,
    sort: Option<String>,
    group: Option<String>,
    #[serde(default)]
    defaults: Attrs,
    #[serde(default)]
    pretty_urls: bool,
}

// longest generated excerpt, in characters
const EXCERPT_LENGTH: usize = 200;

#[derive(Debug, Clone)]
struct Member {
    path: PathBuf,
    outpath: PathBuf,
    url: String,
    // the collection's defaults, then the page's own frontmatter
    attrs: Attrs,
    // the page source after its frontmatter, kept so it's only read once
    content: String,
    // plain-text teaser for listings: `excerpt` frontmatter, or from content
    excerpt: String,
    date: Option<FmDate>,
    title: String,
    prev: Option<usize>,
    next: Option<usize>,
}

#[derive(Debug)]
struct Collection {
    name: String,
    defaults: Attrs,
    // sorted oldest first
    members: Vec<Member>,
}

#[derive(Debug, Default)]
pub struct Collections {
    collections: Vec<Collection>,
    // source path -> (collection index, member index)
    by_path: HashMap<PathBuf, (usize, usize)>,
    drafts: HashSet<PathBuf>,
}

// "2009/04/slug.md" or "slug.md.hbs" -> "2009/04/slug"
fn page_stem(rel_path: &Path) -> PathBuf {
    let name = rel_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let stem = name
        .strip_suffix(".md.hbs")
        .or_else(|| name.strip_suffix(".md"))
        .unwrap_or(&name);
    rel_path.with_file_name(stem)
}

fn is_markdown_page(path: &Path) -> bool {
    let name = path.to_string_lossy();
    name.ends_with(".md") || name.ends_with(".md.hbs")
}

// `excerpt` frontmatter as written, or one made from the page's content
fn member_excerpt(attrs: &Attrs, content: &str) -> String {
    match attrs.get("excerpt").and_then(Value::as_str) {
        Some(s) => s.to_string(),
        None => md::excerpt(content, EXCERPT_LENGTH),
    }
}

// true when frontmatter has a `status` other than `publish`
pub fn has_draft_status(attrs: &Attrs) -> bool {
    attrs
        .get("status")
        .is_some_and(|status| status.as_str() != Some("publish"))
}

// a root-relative URL from a path relative to the output directory
fn url_from(prefix: &str, rel: &Path, trailing_slash: bool) -> String {
    let parts: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();
    let slash = if trailing_slash { "/" } else { "" };
    format!("{}{}{}", prefix, parts.join("/"), slash)
}

impl Collections {
    // finds every collection.yaml under the source directory and reads the
    // frontmatter of each member. returns an empty set if there are none.
    pub fn load(config: &Config) -> anyhow::Result<Collections> {
        let files: Vec<PathBuf> = WalkDir::new(&config.sourcedir)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| !e.is_hidden())
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.into_path())
            .collect();

        let mut specs: Vec<(PathBuf, CollectionSpec, GlobMatcher)> = Vec::new();
        for path in files.iter().filter(|p| p.ends_with("collection.yaml")) {
            let yaml = std::fs::read_to_string(path)?;
            let spec: CollectionSpec = serde_yaml::from_str(&yaml)
                .map_err(|e| anyhow!("{} failed to parse: {}", path.display(), e))?;
            if specs.iter().any(|(_, s, _)| s.name == spec.name) {
                bail!(
                    "{}: collection name '{}' is already used",
                    path.display(),
                    spec.name
                );
            }
            if let Some(sort) = &spec.sort {
                if sort != "date" {
                    warn!(
                        "{}: sort '{}' isn't supported yet, sorting by date",
                        path.display(),
                        sort
                    );
                }
            }
            let matcher = Glob::new(&spec.include)
                .map_err(|e| anyhow!("{}: include: {}", path.display(), e))?
                .compile_matcher();
            let dir = path.parent().unwrap_or(Path::new("")).to_path_buf();
            specs.push((dir, spec, matcher));
        }

        let mut result = Collections::default();
        let mut members: Vec<Vec<Member>> = specs.iter().map(|_| Vec::new()).collect();
        for path in files.iter().filter(|p| is_markdown_page(p)) {
            let found = specs.iter().position(|(dir, _, matcher)| {
                path.strip_prefix(dir)
                    .is_ok_and(|rel| matcher.is_match(rel))
            });
            let Some(index) = found else {
                // a page outside any collection can be a draft too, e.g. a
                // page that lists drafts for review
                let (attrs, _) = md::strip_frontmatter(&read_file_to_string(path)?)?;
                if has_draft_status(&attrs) && !config.drafts {
                    info!("draft, not built: {}", path.display());
                    result.drafts.insert(path.clone());
                }
                continue;
            };
            let spec = &specs[index].1;

            let source = read_file_to_string(path)?;
            let (page_attrs, content) = md::strip_frontmatter(&source)?;
            let mut attrs = spec.defaults.clone();
            attrs.extend(page_attrs);

            if has_draft_status(&attrs) {
                if !config.drafts {
                    info!("draft, not built: {}", path.display());
                    result.drafts.insert(path.clone());
                    continue;
                }
                attrs.insert("draft".into(), true.into());
            }

            let rel = path.strip_prefix(&config.sourcedir)?;
            let (outpath, url) = if spec.pretty_urls {
                let stem = page_stem(rel);
                (
                    config.outdir.join(&stem).join("index.html"),
                    url_from(&config.prefix, &stem, true),
                )
            } else {
                let outpath = crate::web::document::Document::from_path(path).outpath(config)?;
                let url = url_from(&config.prefix, outpath.strip_prefix(&config.outdir)?, false);
                (outpath, url)
            };

            let date = attrs.get("date").and_then(FmDate::from_value);
            if !attrs.contains_key("date") {
                warn!(
                    "'{}': no date, sorted after the dated posts in '{}'",
                    path.display(),
                    spec.name
                );
            }
            let title = attrs
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let excerpt = member_excerpt(&attrs, &content);
            members[index].push(Member {
                path: path.clone(),
                outpath,
                url,
                attrs,
                content,
                excerpt,
                date,
                title,
                prev: None,
                next: None,
            });
        }

        for ((_, spec, _), mut list) in specs.into_iter().zip(members) {
            list.sort_by(|a, b| {
                (a.date.is_none(), &a.date, &a.title, &a.path).cmp(&(
                    b.date.is_none(),
                    &b.date,
                    &b.title,
                    &b.path,
                ))
            });
            link_prev_next(&mut list, spec.group.as_deref());
            let ci = result.collections.len();
            for (mi, m) in list.iter().enumerate() {
                result.by_path.insert(m.path.clone(), (ci, mi));
            }
            info!("collection '{}': {} pages", spec.name, list.len());
            result.collections.push(Collection {
                name: spec.name,
                defaults: spec.defaults,
                members: list,
            });
        }
        Ok(result)
    }

    fn member(&self, path: &Path) -> Option<(&Collection, &Member)> {
        let (ci, mi) = self.by_path.get(path)?;
        let collection = &self.collections[*ci];
        Some((collection, &collection.members[*mi]))
    }

    // true for a page whose status isn't `publish`, unless drafts are built
    pub fn is_draft(&self, path: &Path) -> bool {
        self.drafts.contains(path)
    }

    // where a member is written, which differs from the usual place when
    // the collection has pretty_urls
    pub fn outpath(&self, path: &Path) -> Option<&Path> {
        self.member(path).map(|(_, m)| m.outpath.as_path())
    }

    // frontmatter defaults for a member page
    pub fn defaults(&self, path: &Path) -> Option<&Attrs> {
        self.member(path).map(|(c, _)| &c.defaults)
    }

    // a member's frontmatter (with defaults) and content, as already read
    pub fn source(&self, path: &Path) -> Option<(Attrs, String)> {
        self.member(path)
            .map(|(_, m)| (m.attrs.clone(), m.content.clone()))
    }

    // prev/next/prev-title/next-title for a member page
    pub fn nav_vars(&self, path: &Path) -> Attrs {
        let mut vars = Attrs::new();
        let Some((collection, member)) = self.member(path) else {
            return vars;
        };
        for (key, index) in [("prev", member.prev), ("next", member.next)] {
            if let Some(other) = index.map(|i| &collection.members[i]) {
                vars.insert(key.into(), other.url.clone().into());
                vars.insert(format!("{key}-title"), other.title.clone().into());
            }
        }
        vars
    }

    // for a page with `list` frontmatter: the members it lists, and the
    // same grouped by month, named after the collection (or `as`).
    // `list` may also be a sequence of lists, to show several on one page
    pub fn list_vars(&self, page: &Path, list: &Value) -> Attrs {
        match list {
            Value::Array(lists) => {
                let mut vars = Attrs::new();
                for one in lists {
                    vars.extend(self.one_list_vars(page, one));
                }
                vars
            }
            _ => self.one_list_vars(page, list),
        }
    }

    fn one_list_vars(&self, page: &Path, list: &Value) -> Attrs {
        let mut vars = Attrs::new();
        let empty = serde_json::Map::new();
        let (name, filter, order, var, limit) = match list {
            Value::String(name) => (name.as_str(), &empty, "oldest", name.as_str(), None),
            Value::Object(spec) => {
                let Some(name) = spec.get("collection").and_then(Value::as_str) else {
                    warn!("'{}': list needs a collection name", page.display());
                    return vars;
                };
                let filter = match spec.get("filter") {
                    None => &empty,
                    Some(Value::Object(filter)) => filter,
                    Some(other) => {
                        warn!(
                            "'{}': list filter should be key: value pairs, found {}",
                            page.display(),
                            other
                        );
                        &empty
                    }
                };
                let order = spec
                    .get("order")
                    .and_then(Value::as_str)
                    .unwrap_or("oldest");
                let var = match spec.get("as") {
                    None => name,
                    Some(Value::String(var)) => var.as_str(),
                    Some(other) => {
                        warn!(
                            "'{}': list `as` should be a name, found {}",
                            page.display(),
                            other
                        );
                        name
                    }
                };
                let limit = match spec.get("limit") {
                    None => None,
                    Some(limit) => match limit.as_u64() {
                        Some(n) => Some(n as usize),
                        None => {
                            warn!(
                                "'{}': list limit should be a whole number, found {}",
                                page.display(),
                                limit
                            );
                            None
                        }
                    },
                };
                (name, filter, order, var, limit)
            }
            other => {
                warn!(
                    "'{}': list should be a collection name, found {}",
                    page.display(),
                    other
                );
                return vars;
            }
        };
        let Some(collection) = self.collections.iter().find(|c| c.name == name) else {
            warn!("'{}': no collection named '{}'", page.display(), name);
            return vars;
        };
        if order != "oldest" && order != "newest" {
            warn!(
                "'{}': list order should be oldest or newest, found '{}'",
                page.display(),
                order
            );
        }

        let mut selected: Vec<&Member> = collection
            .members
            .iter()
            .filter(|m| filter.iter().all(|(k, v)| m.attrs.get(k) == Some(v)))
            .collect();
        if order == "newest" {
            selected.reverse();
        }
        if let Some(limit) = limit {
            selected.truncate(limit);
        }

        let item = |m: &Member| {
            let mut attrs = m.attrs.clone();
            attrs.insert("url".into(), m.url.clone().into());
            attrs.insert("excerpt".into(), m.excerpt.clone().into());
            Value::Object(attrs)
        };
        let mut months: Vec<(String, Vec<Value>)> = Vec::new();
        for m in &selected {
            let Some(month) = m.date.as_ref().and_then(FmDate::month) else {
                continue;
            };
            match months.last_mut() {
                Some((last, items)) if *last == month => items.push(item(m)),
                _ => months.push((month, vec![item(m)])),
            }
        }
        let by_month: Vec<Value> = months
            .into_iter()
            .map(|(month, items)| {
                let mut group = Attrs::new();
                group.insert("month".into(), month.into());
                group.insert(var.into(), items.into());
                Value::Object(group)
            })
            .collect();

        vars.insert(
            var.into(),
            selected.into_iter().map(item).collect::<Vec<_>>().into(),
        );
        vars.insert(format!("{var}_by_month"), by_month.into());
        vars
    }
}

// links each member to its neighbors, within its group if there is one
fn link_prev_next(members: &mut [Member], group: Option<&str>) {
    let mut last_in_group: HashMap<Option<String>, usize> = HashMap::new();
    for i in 0..members.len() {
        let key = match group {
            None => None,
            Some(group) => match members[i].attrs.get(group) {
                Some(Value::String(value)) => Some(value.clone()),
                other => {
                    warn!(
                        "'{}': {} should be a string, found {}; no prev/next",
                        members[i].path.display(),
                        group,
                        other.map_or("nothing".to_string(), |v| v.to_string())
                    );
                    continue;
                }
            },
        };
        if let Some(prev) = last_in_group.insert(key, i) {
            members[i].prev = Some(prev);
            members[prev].next = Some(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(path: &str, attrs: &str) -> Member {
        let attrs = md::parse_attrs(attrs).unwrap();
        Member {
            path: PathBuf::from(path),
            outpath: PathBuf::from(path),
            url: format!("/{path}/"),
            date: attrs.get("date").and_then(FmDate::from_value),
            title: String::new(),
            content: String::new(),
            excerpt: format!("about {path}"),
            attrs,
            prev: None,
            next: None,
        }
    }

    #[test]
    fn draft_status_is_anything_but_publish() {
        let draft = |yaml: &str| has_draft_status(&md::parse_attrs(yaml).unwrap());
        assert!(draft("status: draft"));
        assert!(draft("status: pending"));
        assert!(draft("status: [publish]"));
        assert!(!draft("status: publish"));
        assert!(!draft("title: no status"));
    }

    #[test]
    fn page_stem_strips_markdown_extensions() {
        assert_eq!(
            page_stem(Path::new("2009/04/slug.md")),
            PathBuf::from("2009/04/slug")
        );
        assert_eq!(page_stem(Path::new("a.md.hbs")), PathBuf::from("a"));
        assert_eq!(
            page_stem(Path::new("v0.5-release.md")),
            PathBuf::from("v0.5-release")
        );
    }

    #[test]
    fn url_from_relative_path() {
        assert_eq!(
            url_from("/", Path::new("2009/04/slug"), true),
            "/2009/04/slug/"
        );
        assert_eq!(
            url_from("/blog/", Path::new("a.html"), false),
            "/blog/a.html"
        );
    }

    #[test]
    fn prev_next_stays_in_group() {
        let mut list = vec![
            member("a", "category: code"),
            member("b", "category: general"),
            member("c", "category: code"),
            member("d", "category: [code]"),
            member("e", "category: general"),
        ];
        link_prev_next(&mut list, Some("category"));
        let links: Vec<(Option<usize>, Option<usize>)> =
            list.iter().map(|m| (m.prev, m.next)).collect();
        assert_eq!(
            links,
            [
                (None, Some(2)),
                (None, Some(4)),
                (Some(0), None),
                (None, None), // not a string: no prev/next
                (Some(1), None),
            ]
        );
    }

    #[test]
    fn prev_next_without_group_links_all() {
        let mut list = vec![member("a", "x: 1"), member("b", "x: 2")];
        link_prev_next(&mut list, None);
        assert_eq!((list[0].prev, list[0].next), (None, Some(1)));
        assert_eq!((list[1].prev, list[1].next), (Some(0), None));
    }

    fn sample() -> Collections {
        let mut members = vec![
            member("p1", "date: 2009-04-17\ncategory: code"),
            member("p2", "date: 2009-04-18\ncategory: general"),
            member("p3", "date: 2009-05-01\ncategory: code"),
            member("p4", "category: code"),
        ];
        link_prev_next(&mut members, Some("category"));
        let mut result = Collections::default();
        for (i, m) in members.iter().enumerate() {
            result.by_path.insert(m.path.clone(), (0, i));
        }
        result.collections.push(Collection {
            name: "posts".into(),
            defaults: Attrs::new(),
            members,
        });
        result
    }

    #[test]
    fn nav_vars_have_urls_and_titles() {
        let vars = sample().nav_vars(Path::new("p3"));
        assert_eq!(vars.get("prev"), Some(&"/p1/".into()));
        assert_eq!(vars.get("prev-title"), Some(&"".into()));
        assert_eq!(vars.get("next"), Some(&"/p4/".into()));
        assert!(sample().nav_vars(Path::new("not-a-member")).is_empty());
    }

    #[test]
    fn list_vars_filter_order_and_months() {
        let list = md::parse_attrs(
            "list:\n  collection: posts\n  filter:\n    category: code\n  order: newest",
        )
        .unwrap();
        let vars = sample().list_vars(Path::new("archive.md"), &list["list"]);
        let urls: Vec<&str> = vars["posts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["url"].as_str().unwrap())
            .collect();
        assert_eq!(urls, ["/p4/", "/p3/", "/p1/"]);
        let months: Vec<&str> = vars["posts_by_month"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m["month"].as_str().unwrap())
            .collect();
        // undated p4 is listed, but not under a month
        assert_eq!(months, ["2009-05", "2009-04"]);
    }

    #[test]
    fn member_excerpt_frontmatter_wins() {
        let attrs = md::parse_attrs("excerpt: Written by hand").unwrap();
        assert_eq!(
            member_excerpt(&attrs, "First line. Second."),
            "Written by hand"
        );
        assert_eq!(
            member_excerpt(&Attrs::new(), "First line. Second."),
            "First line."
        );
    }

    #[test]
    fn list_vars_items_have_excerpt() {
        let vars = sample().list_vars(Path::new("a.md"), &"posts".into());
        assert_eq!(vars["posts"][0]["excerpt"], "about p1");
        assert_eq!(vars["posts_by_month"][0]["posts"][0]["excerpt"], "about p1");
    }

    #[test]
    fn list_vars_short_form_and_unknown_collection() {
        let vars = sample().list_vars(Path::new("a.md"), &"posts".into());
        assert_eq!(vars["posts"].as_array().unwrap().len(), 4);
        assert_eq!(vars["posts_by_month"].as_array().unwrap().len(), 2);
        assert!(sample()
            .list_vars(Path::new("a.md"), &"nope".into())
            .is_empty());
    }

    fn urls<'a>(vars: &'a Attrs, name: &str) -> Vec<&'a str> {
        vars[name]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["url"].as_str().unwrap())
            .collect()
    }

    #[test]
    fn list_vars_several_named_lists_with_limit() {
        let list = md::parse_attrs(
            "list:
  - collection: posts
    filter: {category: code}
    order: newest
    limit: 2
    as: notebook
  - collection: posts
    filter: {category: general}
    as: recent",
        )
        .unwrap();
        let vars = sample().list_vars(Path::new("index.md"), &list["list"]);
        assert_eq!(urls(&vars, "notebook"), ["/p4/", "/p3/"]);
        assert_eq!(urls(&vars, "recent"), ["/p2/"]);
        assert!(!vars.contains_key("posts"));
        // month groups use the `as` name too
        let months = vars["notebook_by_month"].as_array().unwrap();
        assert_eq!(months.len(), 1);
        assert_eq!(months[0]["notebook"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn list_vars_bad_limit_and_as_are_ignored() {
        let list = md::parse_attrs("list:\n  collection: posts\n  limit: lots\n  as: [x]").unwrap();
        let vars = sample().list_vars(Path::new("a.md"), &list["list"]);
        assert_eq!(vars["posts"].as_array().unwrap().len(), 4);
    }
}
