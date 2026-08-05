use crate::web::read_file_to_string;
use crate::web::words::*;
use cmark::Event;
use pulldown_cmark as cmark;
use std::collections::HashMap;
use std::path::Path;

mod ref_markdown;
pub use ref_markdown::Ref;

// splits optional YAML frontmatter (a `---`-delimited block at the top of the
// file) from the rest of the source, returning the parsed attributes
// alongside the remaining content. Returns an empty map if there's no
// frontmatter, rather than erroring, since frontmatter is optional.
pub(crate) fn strip_frontmatter(source: &str) -> anyhow::Result<(HashMap<String, String>, String)> {
    use matter::matter;
    match matter(source) {
        None => Ok((HashMap::new(), source.to_string())),
        Some((yaml_string, content)) => {
            let attrs: HashMap<String, String> = serde_yaml::from_str(&yaml_string)?;
            Ok((attrs, content))
        }
    }
}

pub fn file2html<P: AsRef<Path>>(
    sourcepath: P,
) -> anyhow::Result<(HashMap<String, String>, Vec<u8>)> {
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
) -> anyhow::Result<(HashMap<String, String>, Vec<u8>)> {
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
        assert_eq!(attrs.get("title"), Some(&"Hello".to_string()));
        assert_eq!(attrs.get("author"), Some(&"Ada".to_string()));
        assert_eq!(content, "hello world");
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
