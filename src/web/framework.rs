use std::io::Write;
use tracing::trace;

use crate::{
    config::Context,
    web::document::HtmlGenerator
};


pub fn render_html<W: Write>(
    context: &Context,
    source: HtmlGenerator,
    output:  &mut W,
) -> anyhow::Result<()> {
    trace!("web::html::render");
    source.render(context, output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::document::Document;

    #[test]
    fn test_render_from_markdown() {
        let mut write_buf: Vec<u8> = Vec::new();

        let config = crate::config::Config::default();
        let mut hbs = handlebars::Handlebars::new();
        let default_tpl = r#"<html><body>{{{ body }}}</body></html>"#;
        hbs.register_template_string("default", default_tpl).unwrap();
        let context = Context {
            config: &config,
            hbs
        };
        let doc = Document::from_path("src/test/data/short-sentence.md");
        let html_source = doc.html_generator(&context).unwrap().unwrap();
        render_html(&context, html_source, &mut write_buf).unwrap();
        let expected = "<html><body><p>it may contain annotations, additions and footnotes</p>\n</body></html>".to_string();
        let output_string: String = String::from_utf8(write_buf).unwrap();
        assert_eq!(output_string, expected);
    }
}