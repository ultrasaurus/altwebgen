use pulldown_cmark as cmark;
use cmark::Event;
use crate::web::read_file_to_string;
use std::path::Path;
use words::WordTime;

mod ref_markdown;
pub use ref_markdown::Ref as Ref;

pub fn file2html<P: AsRef<Path>>(sourcepath: P) -> anyhow::Result<Vec<u8>> {
    let source = read_file_to_string(sourcepath)?;
    str2html(&source)
}

fn str2html(source: &str) -> anyhow::Result<Vec<u8>> {
    let mut html_body: Vec<u8> = Vec::new();

    let parser = cmark::Parser::new(&source);
    cmark::html::write_html(&mut html_body, parser)?;

    Ok(html_body)
}

pub fn file2html_with_timing<P: AsRef<Path>>(md_path: P, transcript_path: P) -> anyhow::Result<Vec<u8>> {
    let md_text = read_file_to_string(md_path)?;
    let file = std::fs::File::open(transcript_path)?;
    let timings = WordTime::from_transcript(file)?;

    str2html_with_timing(&md_text, &timings)
}

fn str2html_with_timing(source: &str, timings: &Vec<WordTime>) -> anyhow::Result<Vec<u8>> {
    let mut html_body = Vec::new();

    let mut new_event_list: Vec<Event> = Vec::new();
    let mut parser = cmark::Parser::new(&source);
    while let Some(event) = parser.next() {
        let next_event= match event {
            Event::Text(cow_str) => {
                let html_buf = words::html_words(&cow_str, Some(timings))?;
                let html_string = String::from(html_buf);
                Event::Html(html_string.into())
            },
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
            WordTime { start_time: 0.9, end_time: 0.1, body: "hello".to_string() },
            WordTime { start_time: 0.2, end_time: 0.3, body: "world".to_string() }
        ];
       let result = str2html_with_timing("hello world", &timings).unwrap();
       let result_string = String::from_utf8(result).unwrap();
        assert_eq!("<p><span word='0' start='0.9' end='0.1' debug_body='hello'>hello</span> <span word='1' start='0.2' end='0.3' debug_body='world'>world</span></p>\n", result_string);

    }

}