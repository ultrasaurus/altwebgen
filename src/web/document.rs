use mime::Mime;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::info;
use crate::config::{Config, Context};
use crate::web::md;
use crate::util::*;

#[derive(Debug, Clone)]
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
            },
            _ => stem.with_extension("html")

        };
        Ok(path)
    }

    pub fn html_generator(&self, context: &Context) -> anyhow::Result<Option<HtmlGenerator>> {
        match HtmlGenerator::from_document(context, self) {
            Err(e) => {
                if e.downcast_ref() == Some(&NotHtmlSourceError {})  {
                    return Ok(None)
                } else {
                    anyhow::bail!(e)
                }
            },
            Ok(generator) => Ok(Some(generator))
        }
    }
}


//-------------errors----------------
#[derive(Debug, Clone, PartialEq)]
pub struct NotHtmlSourceError {  }  //path: String
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
    Html(String)
}

impl HtmlGenerator {
    pub fn from_document(context: &Context, document: &Document) -> anyhow::Result<Self> {

        return match document.mime.subtype().as_str() {
                "x-handlebars-template" => {
                    let data = HandlebarsTemplate::from_path(context, &document.path)?;
                    Ok(HtmlGenerator::Template(data))
                },
                "markdown"
                => {
                    let data = MarkdownData::from_path(context, &document.path)?;
                    Ok(HtmlGenerator::Markdown(data))
                },
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
            HtmlGenerator::Html(html_string) => { writer.write(html_string.as_bytes())?; }
         }
        Ok(())
    }
}
//---------------
use std::collections::HashMap;

// private function
//   reads markdown source file
//   parses yaml front matter into Hashmap of (key, value) pairs
//   returns Hashmap + rest of file
fn read_source<P: AsRef<Path>>(sourcepath: P) -> anyhow::Result<(HashMap<String, String>, String)>
{
    let source = read_file_to_string(sourcepath)?;
    use matter::matter;
    let (data, content) = match matter(&source) {
        None => {info!("matter: None");
            let data: HashMap<String, String> = HashMap::new();
            (data, source)
        },

        Some((yaml_string, content)) => {
            info!("matter:\n{:?}\n------", yaml_string);
            let data:HashMap<String, String> = serde_yaml::from_str(&yaml_string)?;

            //  let data: HashMap<&str, String> = HashMap::new();
            (data, content)
        }
    };
    Ok((data, content))
}

#[derive(PartialEq, Debug, Clone)]
pub struct MarkdownData {
    attr: HashMap<String, String>,
}

trait GenerateHtml {
    fn render<W: Write>(&self, context: &Context, writer: &mut W) -> anyhow::Result<()>;
}

impl GenerateHtml for MarkdownData {
    fn render<W: Write>(&self, context: &Context, writer: &mut W) -> anyhow::Result<()> {
        let layout_key = "layout".to_string();
        let layout_name = if let Some(layout) = self.attr.get(&layout_key) {
            layout.as_str()
        } else {
            "default"
        };
        info!("MarkdownData::render with layout: {layout_name}");
        context.hbs.render_to_write(layout_name, &self.attr, writer)?;
        Ok(())
    }
}

impl MarkdownData {
    fn from_path<P:AsRef<Path>>(context: &Context, path: P) -> anyhow::Result<Self> {
        let (mut template_vars, content) = read_source(path)?;
        let site_attr = context.config.site_attr.clone();
        template_vars.extend(site_attr);

        let html_body= md::str2html(&content)?;
        let body_string = String::from_utf8(html_body)?;

        template_vars.insert("body".into(), body_string);

        Ok(MarkdownData {
            attr: template_vars,
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct HandlebarsTemplate {
    attr: HashMap<String, String>,
    is_markdown: bool
}

impl GenerateHtml for HandlebarsTemplate {
    fn render<W: Write>(&self, context: &Context, writer: &mut W) -> anyhow::Result<()> {
        let layout_key = "layout".to_string();
        let layout_name = if let Some(layout) = self.attr.get(&layout_key) {
            layout.as_str()
        } else {
            "default"
        };
        info!("HandlebarsTemplate::render with layout: {layout_name}");
        context.hbs.render_to_write(layout_name, &self.attr, writer)?;
        Ok(())
    }
}
impl HandlebarsTemplate {
    fn from_path<P:AsRef<Path>>(context: &Context, path: P) -> anyhow::Result<Self> {
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

        let (mut data, content) = read_source(path)?;
        let site_attr = context.config.site_attr.clone();
        data.extend(site_attr);
        let hbs = &context.hbs;
        let mut rendered_body: String = hbs.render_template(&content, &data)?;

        if is_markdown {
            let html_bytes = md::str2html(&rendered_body)?;
            rendered_body = String::from_utf8(html_bytes)?;
        }
        data.insert("body".into(), rendered_body);
        Ok(HandlebarsTemplate {
            attr: data,
            is_markdown
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use handlebars::Handlebars;

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
        hbs.register_template_string("default", default_tpl).unwrap();
        let config = Config::default();
        let context = Context {
            config: &config,
            hbs
        };
        let doc = Document::from_path("src/test/data/short-sentence.md");
        let maybe_html_source: Option<HtmlGenerator> = doc.html_generator(&context).unwrap();
        assert!(maybe_html_source != None);
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
        hbs.register_template_string("default", default_tpl).unwrap();
        let config = Config::default();
        let context = Context {
            config: &config,
            hbs
        };
        let doc = Document::from_path("src/test/data/var.md.hbs");
        let maybe_html_source: Option<HtmlGenerator> = doc.html_generator(&context).unwrap();
        assert!(maybe_html_source != None);
        if let Some(html_source) = maybe_html_source {
            let mut write_buf: Vec<u8> = Vec::new();
            html_source.render(&context, &mut write_buf).unwrap();
            let output_string: String = String::from_utf8(write_buf).unwrap();
            assert_eq!(expected, &output_string)
        }

    }
}