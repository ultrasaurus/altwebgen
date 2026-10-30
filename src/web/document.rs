use crate::config::{Config, Context};
use crate::util::*;
use crate::web::md;
use mime::Mime;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{info, trace};

#[derive(Debug, Clone)]
pub struct Document {
    pub path: PathBuf,
    pub mime: Mime,
}

impl Document {
    pub fn from_path<P: AsRef<Path>>(document_path: P) -> Self {
        let path = document_path.as_ref();
        Document {
            path: PathBuf::from(path),
            mime: {
                match path.mimetype() {
                    Some(mimetype) => mimetype,
                    None => mime::APPLICATION_OCTET_STREAM,
                }
            },
        }
    }
    pub fn outpath(&self, config: &Config) -> anyhow::Result<PathBuf> {
        use std::ffi::OsStr;
        let stem = config.outpath(&self.path)?;
        let path = match self.mime.subtype().as_str() {
            "x-handlebars-template" => {
                let new_path = stem.with_extension("");
                if new_path.extension() == Some(OsStr::new("md")) {
                    new_path.with_extension("html")
                } else {
                    new_path
                }
            }
            _ => stem.with_extension("html"),
        };
        Ok(path)
    }

    pub fn html_generator(
        &self,
        context: &Context,
        collections: &Collections,
    ) -> anyhow::Result<Option<HtmlGenerator>> {
        match HtmlGenerator::from_document(context, collections, self) {
            Err(e) => {
                if e.downcast_ref() == Some(&NotHtmlSourceError {}) {
                    Ok(None)
                } else {
                    anyhow::bail!(e)
                }
            }
            Ok(generator) => Ok(Some(generator)),
        }
    }
}

//-------------errors----------------
#[derive(Debug, Clone, PartialEq)]
pub struct NotHtmlSourceError {} //path: String
impl std::error::Error for NotHtmlSourceError {}
impl fmt::Display for NotHtmlSourceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "not an HTML source, based on path file extension") //: {}", self.path)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum HtmlGenerator {
    Markdown(MarkdownData),
    Template(HandlebarsTemplate),
    Html(String),
}

impl HtmlGenerator {
    pub fn from_document(
        context: &Context,
        collections: &Collections,
        document: &Document,
    ) -> anyhow::Result<Self> {
        match document.mime.subtype().as_str() {
            "x-handlebars-template" => {
                let data = HandlebarsTemplate::from_path(context, collections, &document.path)?;
                Ok(HtmlGenerator::Template(data))
            }
            "markdown" => {
                let data = MarkdownData::from_path(context, collections, &document.path)?;
                Ok(HtmlGenerator::Markdown(data))
            }
            "html" => {
                let html_string = crate::util::read_file_to_string(&document.path)?;
                Ok(HtmlGenerator::Html(html_string))
            }
            _ => {
                //anyhow::bail!(NotHtmlSourceError(document.path.display().to_string()))
                anyhow::bail!(NotHtmlSourceError {})
            }
        }
    }
    pub fn render<W: Write>(&self, context: &Context, writer: &mut W) -> anyhow::Result<()> {
        match self {
            HtmlGenerator::Markdown(md) => md.render(context, writer)?,
            HtmlGenerator::Template(template) => template.render(context, writer)?,
            HtmlGenerator::Html(html_string) => {
                writer.write_all(html_string.as_bytes())?;
            }
        }
        Ok(())
    }
}
//---------------
use crate::web::collection::Collections;
use crate::web::md::Attrs;
use serde_json::Value;
use tracing::warn;

// a source file's YAML frontmatter and content: as read by the collection
// pass for a collection member, otherwise read from the file
fn read_source(collections: &Collections, sourcepath: &Path) -> anyhow::Result<(Attrs, String)> {
    if let Some(source) = collections.source(sourcepath) {
        return Ok(source);
    }
    let source = read_file_to_string(sourcepath)?;
    md::strip_frontmatter(&source)
}

// template variables for one page: site-wide values from _site.yaml, then
// collection defaults, then the page's own frontmatter (so the page wins on
// a shared key), then collection prev/next, any listed collection, and
// series navigation vars
fn page_vars(
    context: &Context,
    collections: &Collections,
    path: &Path,
    page_attrs: Attrs,
) -> Attrs {
    if let Some(date) = page_attrs.get("date") {
        if crate::web::date::FmDate::from_value(date).is_none() {
            warn!(
                "'{}': date {} isn't a valid date (expected e.g. 1984, 1984-10, 2025-11-08), it will display as written",
                path.display(),
                date
            );
        }
    }
    let mut vars = context.config.site_attr.clone();
    if let Some(defaults) = collections.defaults(path) {
        vars.extend(defaults.clone());
    }
    vars.extend(page_attrs);
    vars.extend(collections.nav_vars(path));
    if let Some(list) = vars.get("list").cloned() {
        vars.extend(collections.list_vars(path, &list));
    }
    if let Some(series) = crate::web::series::Series::load_sibling(path) {
        let slug = crate::web::series::page_slug(path);
        vars.extend(
            series
                .nav_vars(&slug)
                .into_iter()
                .map(|(k, v)| (k, Value::String(v))),
        );
    }
    vars
}

// the template named by `layout` frontmatter, or "default"
fn layout_name(attr: &Attrs) -> &str {
    match attr.get("layout") {
        None => "default",
        Some(Value::String(layout)) => layout.as_str(),
        Some(other) => {
            warn!("layout should be a template name, found: {other}, using 'default'");
            "default"
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct MarkdownData {
    attr: Attrs,
}

trait GenerateHtml {
    fn render<W: Write>(&self, context: &Context, writer: &mut W) -> anyhow::Result<()>;
}

impl GenerateHtml for MarkdownData {
    fn render<W: Write>(&self, context: &Context, writer: &mut W) -> anyhow::Result<()> {
        let layout_name = layout_name(&self.attr);
        info!("MarkdownData::render with layout: {layout_name}");
        context
            .hbs
            .render_to_write(layout_name, &self.attr, writer)?;
        Ok(())
    }
}

impl MarkdownData {
    fn from_path<P: AsRef<Path>>(
        context: &Context,
        collections: &Collections,
        path: P,
    ) -> anyhow::Result<Self> {
        let (page_attrs, content) = read_source(collections, path.as_ref())?;
        let mut template_vars = page_vars(context, collections, path.as_ref(), page_attrs);

        let html_body = md::str2html(&content)?;
        let body_string = String::from_utf8(html_body)?;

        template_vars.insert("body".into(), body_string.into());

        Ok(MarkdownData {
            attr: template_vars,
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct HandlebarsTemplate {
    attr: Attrs,
    is_markdown: bool,
}

impl GenerateHtml for HandlebarsTemplate {
    fn render<W: Write>(&self, context: &Context, writer: &mut W) -> anyhow::Result<()> {
        let layout_name = layout_name(&self.attr);
        trace!("HandlebarsTemplate::render with layout: {layout_name}");
        context
            .hbs
            .render_to_write(layout_name, &self.attr, writer)?;
        Ok(())
    }
}
impl HandlebarsTemplate {
    fn from_path<P: AsRef<Path>>(
        context: &Context,
        collections: &Collections,
        path: P,
    ) -> anyhow::Result<Self> {
        // check if target file has markdown, so we can render it as html below
        let mut is_markdown: bool = false;
        if let Some(root_file_name) = path.as_ref().file_stem() {
            let root_file_path = PathBuf::from(root_file_name);
            let maybe_next_mime: Option<Mime> = root_file_path.mimetype();
            if let Some(next_mime) = maybe_next_mime {
                if next_mime.subtype() == "markdown" {
                    is_markdown = true;
                }
            };
        }

        let (page_attrs, content) = read_source(collections, path.as_ref())?;
        let mut data = page_vars(context, collections, path.as_ref(), page_attrs);

        let hbs = &context.hbs;
        let mut rendered_body: String = hbs.render_template(&content, &data)?;

        if is_markdown {
            let html_bytes = md::str2html(&rendered_body)?;
            rendered_body = String::from_utf8(html_bytes)?;
        }
        data.insert("body".into(), rendered_body.into());
        Ok(HandlebarsTemplate {
            attr: data,
            is_markdown,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use handlebars::Handlebars;

    #[test]
    fn test_page_vars_page_wins_over_site() {
        let config = Config {
            site_attr: md::parse_attrs("baseurl: /site\ntitle: Site Title").unwrap(),
            ..Config::default()
        };
        let context = Context {
            config: &config,
            hbs: Handlebars::new(),
            ref_outputs: Vec::new(),
        };
        let page_attrs = md::parse_attrs("title: Page Title").unwrap();
        let vars = page_vars(
            &context,
            &Collections::default(),
            Path::new("src/test/data/short-sentence.md"),
            page_attrs,
        );
        assert_eq!(vars.get("title"), Some(&"Page Title".into()));
        assert_eq!(vars.get("baseurl"), Some(&"/site".into()));
    }

    #[test]
    fn test_layout_name() {
        assert_eq!(layout_name(&Attrs::new()), "default");
        assert_eq!(
            layout_name(&md::parse_attrs("layout: booklet").unwrap()),
            "booklet"
        );
        assert_eq!(
            layout_name(&md::parse_attrs("layout: [a, b]").unwrap()),
            "default"
        );
    }

    #[test]
    fn test_outpath_from_md() {
        let config = Config::default();
        let doc = Document::from_path("source/foo.md");
        let outpath = doc.outpath(&config).unwrap();
        let outpath_str = outpath.to_str().unwrap();
        assert_eq!(outpath_str, ".dist/foo.html");
    }

    #[test]
    fn test_outpath_from_md_hbs() {
        let config = Config::default();
        let doc = Document::from_path("source/foo.md.hbs");
        let outpath = doc.outpath(&config).unwrap();
        let outpath_str = outpath.to_str().unwrap();
        assert_eq!(outpath_str, ".dist/foo.html");
    }

    #[test]
    fn test_html_gen_markdown() {
        let default_tpl = r#"<html><body>{{{ body }}}</body></html>"#;
        let expected = "<html><body><p>it may contain annotations, additions and footnotes</p>\n</body></html>";
        let mut hbs = Handlebars::new();
        hbs.register_template_string("default", default_tpl)
            .unwrap();
        let config = Config::default();
        let context = Context {
            config: &config,
            hbs,
            ref_outputs: Vec::new(),
        };
        let doc = Document::from_path("src/test/data/short-sentence.md");
        let maybe_html_source: Option<HtmlGenerator> = doc
            .html_generator(&context, &Collections::default())
            .unwrap();
        assert!(maybe_html_source.is_some());
        if let Some(html_source) = maybe_html_source {
            let mut write_buf: Vec<u8> = Vec::new();
            html_source.render(&context, &mut write_buf).unwrap();
            let output_string: String = String::from_utf8(write_buf).unwrap();
            assert_eq!(expected, &output_string)
        }
    }

    #[test]
    fn test_html_gen_markdown_hbs() {
        let default_tpl = r#"<html><body>{{{ body }}}</body></html>"#;
        let expected = "<html><body><p>Those who have learned to walk on the threshold of the unknown worlds, by means of what are commonly termed par\nexcellence the exact sciences, may then, with the fair white wings of imagination, hope to soar further into the\nunexplored amidst which we live. -- Ada Lovelace</p>\n</body></html>";
        let mut hbs = Handlebars::new();
        hbs.register_template_string("default", default_tpl)
            .unwrap();
        let config = Config::default();
        let context = Context {
            config: &config,
            hbs,
            ref_outputs: Vec::new(),
        };
        let doc = Document::from_path("src/test/data/var.md.hbs");
        let maybe_html_source: Option<HtmlGenerator> = doc
            .html_generator(&context, &Collections::default())
            .unwrap();
        assert!(maybe_html_source.is_some());
        if let Some(html_source) = maybe_html_source {
            let mut write_buf: Vec<u8> = Vec::new();
            html_source.render(&context, &mut write_buf).unwrap();
            let output_string: String = String::from_utf8(write_buf).unwrap();
            assert_eq!(expected, &output_string)
        }
    }
}
