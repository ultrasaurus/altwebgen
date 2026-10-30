use crate::web::read_file_to_string;
use crate::web::words::*;
use cmark::Event;
use pulldown_cmark as cmark;
use std::path::Path;

mod ref_markdown;
pub use ref_markdown::Ref;

// template variables: frontmatter (or _site.yaml) keys mapped to any YAML
// value (string, number, bool, list, map), held as JSON values since that's
// what handlebars renders from
pub type Attrs = serde_json::Map<String, serde_json::Value>;

// parses a YAML mapping into Attrs. an empty document (e.g. a frontmatter
// block with nothing in it) is an empty map; anything other than a mapping
// at the top level is an error.
pub(crate) fn parse_attrs(yaml: &str) -> anyhow::Result<Attrs> {
    let value: serde_yaml::Value = serde_yaml::from_str(yaml)?;
    match serde_json::to_value(value)? {
        serde_json::Value::Null => Ok(Attrs::new()),
        serde_json::Value::Object(map) => Ok(map),
        other => anyhow::bail!("expected key: value pairs, found: {}", other),
    }
}

// splits optional YAML frontmatter (a `---`-delimited block at the top of the
// file) from the rest of the source, returning the parsed attributes
// alongside the remaining content. Returns an empty map if there's no
// frontmatter, rather than erroring, since frontmatter is optional.
pub(crate) fn strip_frontmatter(source: &str) -> anyhow::Result<(Attrs, String)> {
    use matter::matter;
    match matter(source) {
        None => Ok((Attrs::new(), source.to_string())),
        Some((yaml_string, content)) => Ok((parse_attrs(&yaml_string)?, content)),
    }
}

pub fn file2html<P: AsRef<Path>>(sourcepath: P) -> anyhow::Result<(Attrs, Vec<u8>)> {
    let source = read_file_to_string(sourcepath)?;
    let (attrs, content) = strip_frontmatter(&source)?;
    Ok((attrs, str2html(&content)?))
}

pub fn str2html(source: &str) -> anyhow::Result<Vec<u8>> {
    let mut html_body: Vec<u8> = Vec::new();

    let parser = cmark::Parser::new(source);
    cmark::html::write_html(&mut html_body, parser)?;

    Ok(html_body)
}

pub fn file2html_with_timing<P: AsRef<Path>>(
    md_path: P,
    transcript_path: P,
) -> anyhow::Result<(Attrs, Vec<u8>)> {
    let md_source = read_file_to_string(md_path)?;
    let (attrs, md_text) = strip_frontmatter(&md_source)?;
    let file = std::fs::File::open(transcript_path)?;
    let timings = WordTime::from_transcript(file)?;

    Ok((attrs, str2html_with_timing(&md_text, &timings)?))
}

fn str2html_with_timing(source: &str, timings: &Vec<WordTime>) -> anyhow::Result<Vec<u8>> {
    let mut html_body = Vec::new();
    let mut next_timings: &[WordTime] = timings;
    let mut new_event_list: Vec<Event> = Vec::new();
    let parser = cmark::Parser::new(source);
    for event in parser {
        let next_event = match event {
            Event::Text(cow_str) => {
                let data = html_words(&cow_str, Some(next_timings))?;
                next_timings = &timings[data.last_timing_index..timings.len()];
                Event::Html(data.html.into())
            }
            _ => event,
        };
        new_event_list.push(next_event);
    }

    cmark::html::write_html(&mut html_body, new_event_list.into_iter())?;

    Ok(html_body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_frontmatter_no_frontmatter() {
        let (attrs, content) = strip_frontmatter("hello world").unwrap();
        assert!(attrs.is_empty());
        assert_eq!(content, "hello world");
    }

    #[test]
    fn strip_frontmatter_with_frontmatter() {
        let source = "---\ntitle: Hello\nauthor: Ada\n---\nhello world";
        let (attrs, content) = strip_frontmatter(source).unwrap();
        assert_eq!(attrs.get("title"), Some(&"Hello".into()));
        assert_eq!(attrs.get("author"), Some(&"Ada".into()));
        assert_eq!(content, "hello world");
    }

    #[test]
    fn strip_frontmatter_typed_values() {
        let source = "---\ncategory:\n  - code\nid: 11\ndraft: false\nintro: |\n  line one\n  line two\n---\nbody";
        let (attrs, _) = strip_frontmatter(source).unwrap();
        assert_eq!(attrs.get("category"), Some(&serde_json::json!(["code"])));
        assert_eq!(attrs.get("id"), Some(&serde_json::json!(11)));
        assert_eq!(attrs.get("draft"), Some(&serde_json::json!(false)));
        // the frontmatter block is trimmed before parsing, so the last line
        // of a multi-line string has no trailing newline
        assert_eq!(
            attrs.get("intro"),
            Some(&serde_json::json!("line one\nline two"))
        );
    }

    #[test]
    fn parse_attrs_empty_is_empty_map() {
        assert!(parse_attrs("").unwrap().is_empty());
    }

    #[test]
    fn parse_attrs_rejects_non_mapping() {
        assert!(parse_attrs("- just\n- a list").is_err());
    }

    #[test]
    fn str2html_empty() {
        let result = str2html("").unwrap();
        let result_string = String::from_utf8(result).unwrap();
        assert_eq!("", result_string);
    }

    #[test]
    fn str2html_simple_text() {
        let result = str2html("hello world").unwrap();
        let result_string = String::from_utf8(result).unwrap();
        assert_eq!("<p>hello world</p>\n", result_string);
    }

    #[test]
    fn str2html_with_timing_phrase() {
        let timings = vec![
            WordTime {
                start_time: 0.9,
                end_time: 0.1,
                body: "hello".to_string(),
            },
            WordTime {
                start_time: 0.2,
                end_time: 0.3,
                body: "world".to_string(),
            },
        ];
        let result = str2html_with_timing("hello world", &timings).unwrap();
        let result_string = String::from_utf8(result).unwrap();
        assert_eq!("<p><span word='0' start='0.9' end='0.1' debug_body='hello'>hello</span> <span word='1' start='0.2' end='0.3' debug_body='world'>world</span></p>\n", result_string);
    }
}
