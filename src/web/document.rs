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
        let stem = config.outpath(&self.path)?;
        let path = match self.mime.subtype().as_str() {
            "x-handlebars-template" => stem.with_extension(""),
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
        context.hbs.render_to_write("default", &self.attr, writer)?;
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
    attr: HashMap<String, String>
}

impl GenerateHtml for HandlebarsTemplate {
    fn render<W: Write>(&self, context: &Context, writer: &mut W) -> anyhow::Result<()> {
        context.hbs.render_to_write("default", &self.attr, writer)?;
        Ok(())
    }
}
impl HandlebarsTemplate {
    fn from_path<P:AsRef<Path>>(context: &Context, path: P) -> anyhow::Result<Self> {
        let (mut data, content) = read_source(path)?;
        let site_attr = context.config.site_attr.clone();
        data.extend(site_attr);
        let hbs = &context.hbs;
        let html_body: String = hbs.render_template(&content, &data)?;
        data.insert("body".into(), html_body);

        Ok(HandlebarsTemplate {
            attr: data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use handlebars::Handlebars;

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

}