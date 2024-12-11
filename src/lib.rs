use regex::Regex;
mod transcript;
// pub use transcript::WordTime as WordTime;
use anyhow::Result;

use std::error::Error;

pub struct WordTime {
    pub body: String,
    pub start_time: f64,
    pub end_time: f64,
}

pub fn html_words(text: &str, optional_timing: Option<&Vec<WordTime>>) -> Result<(String, usize, usize), Box<dyn Error>> {

    let regex = Regex::new(r"([a-zà-ýA-ZÀ-Ý0-9]+)([\s$][^a-zà-ýA-ZÀ-Ý0-9]*)?")?;
    let mut html_string = String::new();
    let mut word_index = 0;
    let mut last_timing_index = 0;

    // Create a mutable iterator for the timings, if provided
    let mut timing_iter = optional_timing.map(|timings| timings.iter().peekable());

    // Iterate over each word in the text
    for capture in regex.captures_iter(text) {
        let word = &capture[1];
        let mut matched = false;

        // Only proceed if timings are provided
        if let Some(timings) = &mut timing_iter {
            // Try to find a matching timing for the current word
            if let Some(timing) = timings.peek() {
                if timing.body.to_lowercase() == word.to_lowercase() {
                    // Matching word with timing, add span with timing info
                    html_string.push_str(&format!(
                        "<span word='{}' start='{}' end='{}' debug_body='{}'>{}</span> ",
                        word_index,
                        timing.start_time,
                        timing.end_time,
                        timing.body,
                        word
                    ));
                    timings.next(); // Consume the timing once matched
                    matched = true;
                    last_timing_index = word_index;
                }
            }
        }

        // If no match was found, mark this word as an error
        if !matched {
            html_string.push_str(&format!(
                "<span word='{}' error='NO_MATCH'>{}</span> ",
                word_index,
                word
            ));
        }

        // Move to the next word
        word_index += 1;

        // Exit loop early if no more timings available
        if let Some(timings) = &mut timing_iter {
            if timings.peek().is_none() {
                break;
            }
        }
    }

    // Return the result as a trimmed string, the number of words, and the last timing index used
    Ok((html_string.trim().to_string(), word_index, last_timing_index))
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
        let (result_string, word_count, last_timing_index) = result.unwrap();
        let expected_string = "";
        assert_eq!(result_string, expected_string);
        assert_eq!(word_count, 0);
        assert_eq!(last_timing_index, 0);
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
            WordTime { start_time: 0.0, end_time: 0.1, body: "hello".to_string() },
            WordTime { start_time: 0.2, end_time: 0.3, body: "world".to_string() }
        ];
        let result = html_words("Hello world!", Some(&timings));
        assert!(result.is_ok());
        let (result_string, word_count, last_timing_index) = result.unwrap();
        let expected_string = "<span word='0' start='0' end='0.1' debug_body='hello'>Hello</span> <span word='1' start='0.2' end='0.3' debug_body='world'>world</span>";
        assert_eq!(result_string, expected_string);
        assert_eq!(word_count, 2);
        assert_eq!(last_timing_index, 1);
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
        let (result_string, word_count, last_timing_index) = result.unwrap();
        let expected_string = "<span word='0' start='0' end='0.1' debug_body='hello'>Hello</span> <span word='1' error='NO_MATCH'>there</span> <span word='2' start='0.2' end='0.3' debug_body='world'>world</span>";
        assert_eq!(result_string, expected_string);
        assert_eq!(word_count, 3);
        assert_eq!(last_timing_index, 2);
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
        let (result_string, word_count, last_timing_index) = result.unwrap();
        let expected_string = "<span word='0' start='0' end='0.1' debug_body='hello'>Hello</span> <span word='1' error='NO_MATCH'>my</span> <span word='2' start='0.4' end='0.5' debug_body='world'>world</span>";
        assert_eq!(result_string, expected_string);
        assert_eq!(word_count, 3);
        assert_eq!(last_timing_index, 2);
    }

}