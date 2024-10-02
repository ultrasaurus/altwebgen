use pulldown_cmark as cmark;
use cmark::Event;
use crate::web::read_file_to_string;
use std::path::Path;
mod cowstr;
pub use self::cowstr::CowStrExt;

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

fn str2html_with_timing(source: &str) -> anyhow::Result<Vec<u8>> {
    let mut html_body = Vec::new();

    let mut new_event_list: Vec<Event> = Vec::new();
    let mut parser = cmark::Parser::new(&source);
    while let Some(event) = parser.next() {
        let next_event= match event {
            Event::Text(cow_str) => {
                // let html_string = "";
                // Event::Text(html_string.into())
                Event::Text(cow_str)
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

}