use kuchikiki::traits::TendrilSink;
use std::io::Write;
use tracing::trace;

use crate::{
    config::*,
    devserve::LIVE_RELOAD_JS,
    util::NodeRefExt,
    web::document::HtmlGenerator
};


pub fn render_html<W: Write>(
    context: &Context,
    source: HtmlGenerator,
    output:  &mut W,
) -> anyhow::Result<()> {
    trace!("web::html::render");

    if context.config.mode == Mode::Dev {
        let mut write_buf: Vec<u8> = Vec::new();
        source.render(context, &mut write_buf)?;
        let output_string: String = String::from_utf8(write_buf).unwrap();
        let document = kuchikiki::parse_html().one(output_string);
        document.inject_script(LIVE_RELOAD_JS)?;
        document.serialize( output)?;
    } else {
        source.render(context, output)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        web::document::Document,
        config::Config,
    };
    fn dev_config() -> Config {
        let mut config = Config::default();
        config.mode = Mode::Dev;
        config
    }

    fn context_with_default_template<'a>(config: &'a Config, tmpl: &str) -> Context<'a> {
        let mut hbs = handlebars::Handlebars::new();
        hbs.register_template_string("default", tmpl).unwrap();
        Context {
            config: config,
            hbs
        }
    }

    #[test]
    fn test_render_build_from_markdown() {
        let mut write_buf: Vec<u8> = Vec::new();

        let config = crate::config::Config::default();
        let default_tpl: &str = r#"<html><body>{{{ body }}}</body></html>"#;
        let context = context_with_default_template(&config, default_tpl);

        let doc = Document::from_path("src/test/data/short-sentence.md");
        let html_source = doc.html_generator(&context).unwrap().unwrap();
        render_html(&context, html_source, &mut write_buf).unwrap();
        let expected = "<html><body><p>it may contain annotations, additions and footnotes</p>\n</body></html>".to_string();
        let output_string: String = String::from_utf8(write_buf).unwrap();
        assert_eq!(output_string, expected);
    }

        #[test]
    fn test_render_dev_from_markdown() {
        let mut write_buf: Vec<u8> = Vec::new();

        let config = dev_config();
        let default_tpl: &str = r#"<html><head><title>Test</title></head><body>{{{ body }}}</body></html>"#;
        let context = context_with_default_template(&config, default_tpl);

        let doc = Document::from_path("src/test/data/short-sentence.md");
        let html_source = doc.html_generator(&context).unwrap().unwrap();
        render_html(&context, html_source, &mut write_buf).unwrap();
        let expected: String = format!{
            "<html><head><title>Test</title><script>{}</script></head><body><p>it may contain annotations, additions and footnotes</p>\n</body></html>",
            LIVE_RELOAD_JS
        };

        let output_string: String = String::from_utf8(write_buf).unwrap();
        assert_eq!(output_string, expected);
    }


    #[test]
    fn test_render_dev_from_markdown_no_head() {
        let mut write_buf: Vec<u8> = Vec::new();

        let config = dev_config();
        let default_tpl: &str = r#"<html><body>{{{ body }}}</body></html>"#;
        let context = context_with_default_template(&config, default_tpl);

        let doc = Document::from_path("src/test/data/short-sentence.md");
        let html_source = doc.html_generator(&context).unwrap().unwrap();
        render_html(&context, html_source, &mut write_buf).unwrap();
        let expected: String = format!{
            "<html><head><script>{}</script></head><body><p>it may contain annotations, additions and footnotes</p>\n</body></html>",
            LIVE_RELOAD_JS
        };
        let output_string: String = String::from_utf8(write_buf).unwrap();
        assert_eq!(output_string, expected);
    }

}