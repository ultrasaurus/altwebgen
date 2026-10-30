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

// a plain-text teaser for a page: its first paragraph with words in it
// (skipping ones that are only an image, embed or HTML), cut at the end of
// the first sentence if that comes within `max` characters, otherwise at
// the last word before `max`, with "…"
pub fn excerpt(source: &str, max: usize) -> String {
    use cmark::{Tag, TagEnd};
    let mut text = String::new();
    let mut in_paragraph = false;
    let mut in_image = 0;
    for event in cmark::Parser::new(source) {
        match event {
            Event::Start(Tag::Paragraph) => {
                in_paragraph = true;
                text.clear();
            }
            Event::End(TagEnd::Paragraph) => {
                in_paragraph = false;
                if text.chars().any(char::is_alphanumeric) {
                    break;
                }
                text.clear();
            }
            Event::Start(Tag::Image { .. }) => in_image += 1,
            Event::End(TagEnd::Image) => in_image -= 1,
            Event::Text(s) | Event::Code(s) if in_paragraph && in_image == 0 => text.push_str(&s),
            Event::SoftBreak | Event::HardBreak if in_paragraph => text.push(' '),
            _ => {}
        }
    }
    if !text.chars().any(char::is_alphanumeric) {
        return String::new();
    }
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    shorten(&text, max)
}

// words ending in "." that don't end a sentence
const ABBREVIATIONS: [&str; 8] = ["mr", "mrs", "ms", "dr", "st", "vs", "e.g", "i.e"];

fn shorten(text: &str, max: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    // the first sentence, if it ends within max: . ! or ? then a space
    // (allowing closing quotes or brackets in between), but not after an
    // initial ("B. J. Fogg") or a common abbreviation
    for (i, c) in chars.iter().enumerate().take(max) {
        if !matches!(c, '.' | '!' | '?') {
            continue;
        }
        let mut end = i;
        while chars.get(end + 1).is_some_and(|n| "”’\")]".contains(*n)) {
            end += 1;
        }
        if !chars.get(end + 1).is_none_or(|n| n.is_whitespace()) {
            continue;
        }
        if *c == '.' {
            let start = chars[..i]
                .iter()
                .rposition(|c| c.is_whitespace())
                .map_or(0, |p| p + 1);
            let word: String = chars[start..i].iter().collect::<String>().to_lowercase();
            if word.chars().count() == 1 || ABBREVIATIONS.contains(&word.as_str()) {
                continue;
            }
        }
        if end < max {
            return chars[..=end].iter().collect();
        }
    }
    if chars.len() <= max {
        return text.to_string();
    }
    let cut = chars[..max]
        .iter()
        .rposition(|c| c.is_whitespace())
        .unwrap_or(max);
    let head: String = chars[..cut].iter().collect();
    format!(
        "{}…",
        head.trim_end_matches(|c: char| c.is_whitespace() || ",;:-–—".contains(c))
    )
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
    fn excerpt_first_sentence() {
        let source = "Hello *there*, [friend](http://example.com). More text.\n\nNext paragraph.";
        assert_eq!(excerpt(source, 200), "Hello there, friend.");
    }

    #[test]
    fn excerpt_skips_image_and_html_paragraphs() {
        let source =
            "![a photo](p.jpg)\n\n<figure>\n\n![x](y.png)\n\n</figure>\n\nThe `real` start";
        assert_eq!(excerpt(source, 200), "The real start");
    }

    #[test]
    fn excerpt_cuts_at_a_word() {
        let source = "one two three four five six, seven";
        assert_eq!(excerpt(source, 26), "one two three four five…");
        assert_eq!(excerpt(source, 100), "one two three four five six, seven");
    }

    #[test]
    fn excerpt_sentence_past_max_is_cut() {
        // a full stop after max doesn't count, so cut at a word instead
        assert_eq!(excerpt("A long sentence here. Short.", 10), "A long…");
    }

    #[test]
    fn excerpt_sentence_rules() {
        // an ellipsis doesn't end a sentence
        assert_eq!(
            excerpt("I got to the limit … then I rested.", 200),
            "I got to the limit … then I rested."
        );
        // a closing quote after the full stop is kept
        assert_eq!(
            excerpt("“You can do it.” In a podcast, B. J. said so.", 200),
            "“You can do it.”"
        );
        // initials and abbreviations don't end it
        assert_eq!(
            excerpt("A talk by B. J. Fogg, e.g. this one. More.", 200),
            "A talk by B. J. Fogg, e.g. this one."
        );
    }

    #[test]
    fn excerpt_empty_without_words() {
        assert_eq!(excerpt("![only](an-image.png)", 200), "");
        assert_eq!(excerpt("", 200), "");
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
