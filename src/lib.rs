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
fn html_words(text: &str, optional_timing: Option<&Vec<WordTime>>) -> Result<(String, usize, usize), Box<dyn Error>> {
    let regex = Regex::new(r"([a-zà-ýA-ZÀ-Ý0-9]+)([\s$][^a-zà-ýA-ZÀ-Ý0-9]*)?")?;
    let mut html_string = String::new();
    let mut word_index = 0;
    let mut last_timing_index = 0;

    // Extract timings if provided
    let binding = vec![];
    let timings = optional_timing.unwrap_or(&binding);

    // Iterate over each word in the text
    for capture in regex.captures_iter(text) {
        let word = &capture[1];
        let mut matched = false;

        // Try to find a match with any remaining timing
        for timing_index in last_timing_index..timings.len() {
            let timing = &timings[timing_index];
            println!(
                "Comparing: timing body '{}' with word '{}'",
                timing.body.to_lowercase(),
                word.to_lowercase()
            );

            if timing.body.to_lowercase() == word.to_lowercase() {
                // Match found
                html_string.push_str(&format!(
                    "<span word='{}' start='{}' end='{}' debug_body='{}'>{}</span> ",
                    word_index,
                    timing.start_time,
                    timing.end_time,
                    timing.body,
                    word
                ));
                last_timing_index = timing_index + 1; // Advance timing index only on match
                matched = true;
                break;
            }
        }

        // If no match was found, add error span
        if !matched {
            html_string.push_str(&format!(
                "<span word='{}' error='NO_MATCH'>{}</span> ",
                word_index,
                word
            ));
        }

        word_index += 1;
    }

    // Return the result as a trimmed string, the number of words, and the last timing index used
    Ok((html_string.trim().to_string(), word_index, last_timing_index))
}



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
        assert_eq!(last_timing_index, 2);
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
        assert_eq!(last_timing_index, 3);
    }

}