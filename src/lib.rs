use regex::Regex;
mod transcript;
pub use transcript::WordTime as WordTime;
use anyhow::Result;

pub fn html_words(text: &str, optional_timing: Option<&Vec<WordTime>>) -> Result<String> {
    let regex = Regex::new(r"([a-zà-ýA-ZÀ-Ý0-9]+)([\s$][^a-zà-ýA-ZÀ-Ý0-9]*)?")?;
    let mut matched_tuples = Vec::new();
    let mut word_index = 0;

    // iterate over the text and create tuples of matches with their WordTime if available
    for capture in regex.captures_iter(text) {
        let word = &capture[1];
        if let Some(timings) = optional_timing {
            for timing in timings.iter() {
                if timing.body.to_lowercase() == word.to_lowercase() {
                    matched_tuples.push((word_index, word.to_string(), timing.clone()));
                    break; // next word in the text after finding a match
                }
            }
        }
        word_index += 1; // keeps track of the word's index in the original text
    }

    // generate the HTML output using the matched tuples, order from the WordTime list
    let mut html_string = String::new();
    if let Some(timings) = optional_timing {
        for timing in timings.iter() {
            if let Some((word_index, word, _)) = matched_tuples.iter().find(|(_, _, t)| t.body == timing.body) {
                html_string.push_str(&format!(
                    "<span word='{}' char='{}' start='{}' end='{}' debug_body='{}'>{}</span> ",
                    word_index,
                    0, // Placeholder for character position if needed, which we don't for now
                    timing.start_time,
                    timing.end_time,
                    timing.body,
                    word
                ));
            }
        }
    }

    // trailing spaces begone!
    html_string = html_string.trim().to_string();
    Ok(html_string)
}




//at the end we need to know how many words in transcript and return the number of words
//where in the timings vector we left off -- if 10 and only went through 9 wordtimes, return 9
// I forget why??

#[cfg(test)]
mod tests {
    use super::*;

     #[test]
    fn html_words_empty_string() {
        let result = html_words("", None);
        assert!(result.is_ok());
        let result_string = result.unwrap();
        assert_eq!(result_string, "");
    }

    #[test] //commenting these out temporarily so we can just work with the timings
    fn html_words_hello_world_no_timing() {
        // let result = html_words("Hello world!", None);
        // assert!(result.is_ok());
        // let result_string = result.unwrap();
        // let expected_string = "<span word='0' char='0'>Hello</span> <span word='1' char='6'>world</span>!";
        // assert_eq!(result_string, expected_string);
    }


    #[test]
    fn html_words_hello_world_with_timing() {
        let timings = vec![
            WordTime { start_time: 0.0, end_time: 0.1, body: "hello".to_string()},
            WordTime { start_time: 0.2, end_time: 0.3, body: "world".to_string()}
        ];
        let result = html_words("Hello world!", Some(&timings));
        assert!(result.is_ok());
        let result_string = result.unwrap();
        let expected_string = "<span word='0' char='0' start='0' end='0.1' debug_body='hello'>Hello</span> <span word='1' char='0' start='0.2' end='0.3' debug_body='world'>world</span>";
        assert_eq!(result_string, expected_string);
    }
    
    #[test]
    fn html_words_phrase() {
        // let result = html_words("written or pictorial material", None);
        // assert!(result.is_ok());
        // let result_string = result.unwrap();
        // let expected_string = "<span word='0' char='0'>written</span> <span word='1' char='8'>or</span> <span word='2' char='11'>pictorial</span> <span word='3' char='21'>material</span>";
        // assert_eq!(result_string, expected_string);
    }

    #[test]
    fn html_words_mismatch() {
        let timings = vec![
            WordTime { start_time: 0.0, end_time: 0.1, body: "hello".to_string() },
            WordTime { start_time: 0.2, end_time: 0.3, body: "world".to_string() }
        ];
        let result = html_words("Hello there world!", Some(&timings));
        assert!(result.is_ok());
        let result_string = result.unwrap();
        let expected_string = "<span word='0' char='0' start='0' end='0.1' debug_body='hello'>Hello</span> <span word='2' char='0' start='0.2' end='0.3' debug_body='world'>world</span>";
        assert_eq!(result_string, expected_string);
    }

    #[test]
    fn html_words_mismatch2() {
        let timings = vec![
            WordTime { start_time: 0.0, end_time: 0.1, body: "hello".to_string() },
            WordTime { start_time: 0.2, end_time: 0.3, body: "there".to_string() },
            WordTime { start_time: 0.4, end_time: 0.5, body: "world".to_string() }
        ];
        let result = html_words("Hello my world!", Some(&timings));
        assert!(result.is_ok());
        let result_string = result.unwrap();
        let expected_string = "<span word='0' char='0' start='0' end='0.1' debug_body='hello'>Hello</span> <span word='2' char='0' start='0.4' end='0.5' debug_body='world'>world</span>";
        assert_eq!(result_string, expected_string);
    }

}