use regex::Regex;
mod transcript;
pub use transcript::WordTime as WordTime;

pub fn html_words<S: AsRef<str>>(text: S, optional_timing: Option<&Vec<WordTime>>) -> anyhow::Result<String> {
    let regex = Regex::new(r"([a-zà-ýA-ZÀ-Ý0-9]+)([[\s$][^a-zà-ýA-ZÀ-Ý0-9]]?)")?;
    let mut nth_word = 0;
    let mut html_string = String::new();
    let mut timing_index = 0;
    let mut char_index = 0;

    for match_capture in regex.captures_iter(text.as_ref()) {
        let range: std::ops::Range<usize> = match_capture.get(0).unwrap().range();
        let word = &match_capture[1];

        if let Some(timings) = &optional_timing {
            // Skip words that don't match any timing
            while timing_index < timings.len() && timings[timing_index].body.to_lowercase() != word.to_lowercase() {
                timing_index += 1;
            }
            if timing_index < timings.len() && timings[timing_index].body.to_lowercase() == word.to_lowercase() {
                let word_timing = &timings[timing_index];
                let timing_string = format!(" start='{}' end='{}' debug_body='{}'", word_timing.start_time, word_timing.end_time, word_timing.body);
                html_string.push_str(&format!("<span word='{}' char='{}'{}>{}</span>{}", nth_word, char_index, timing_string, word, &match_capture[2]));
                timing_index += 1;
            } else {
                html_string.push_str(&format!("<span word='{}' char='{}'>{}</span>{}", nth_word, char_index, word, &match_capture[2]));
            }
        } else {
            html_string.push_str(&format!("<span word='{}' char='{}'>{}</span>{}", nth_word, char_index, word, &match_capture[2]));
        }
        nth_word += 1;
        char_index = range.end;
    }

    Ok(html_string)
}

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

    #[test]
    fn html_words_hello_world_no_timing() {
        let result = html_words("Hello world!", None);
        assert!(result.is_ok());
        let result_string = result.unwrap();
        let expected_string = "<span word='0' char='0'>Hello</span> <span word='1' char='6'>world</span>!";
        assert_eq!(result_string, expected_string);
    }

    #[test]
    fn html_words_hello_world_with_timing() {
        let timings = vec![
            WordTime { start_time: 0.0, end_time: 0.1, body: "hello".to_string() },
            WordTime { start_time: 0.2, end_time: 0.3, body: "world".to_string() }
        ];
        let result = html_words("Hello world!", Some(&timings));
        assert!(result.is_ok());
        let result_string = result.unwrap();
        let expected_string = "<span word='0' char='0' start='0' end='0.1' debug_body='hello'>Hello</span> <span word='1' char='6' start='0.2' end='0.3' debug_body='world'>world</span>!";
        assert_eq!(result_string, expected_string);
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
        let expected_string = "<span word='0' char='0' start='0' end='0.1' debug_body='hello'>Hello</span> <span word='2' char='12' start='0.2' end='0.3' debug_body='world'>world</span>!";
        assert_eq!(result_string, expected_string);
    }

    #[test]
    fn html_words_mismatch2() {
        let timings = vec![
            WordTime { start_time: 0.0, end_time: 0.1, body: "hello".to_string() },
            WordTime { start_time: 0.2, end_time: 0.3, body: "there".to_string() },
            WordTime { start_time: 0.4, end_time: 0.5, body: "world".to_string() }
        ];
        let result = html_words("Hello world!", Some(&timings));
        assert!(result.is_ok());
        let result_string = result.unwrap();
        let expected_string = "<span word='0' char='0' start='0' end='0.1' debug_body='hello'>Hello</span> <span word='1' char='6' start='0.4' end='0.5' debug_body='world'>world</span>!";
        assert_eq!(result_string, expected_string);
    }

    #[test]
    fn html_words_mismatch3() {
        let timings = vec![
            WordTime { start_time: 0.0, end_time: 0.1, body: "ban".to_string() },
            WordTime { start_time: 0.2, end_time: 0.3, body: "and".to_string() },
            WordTime { start_time: 0.4, end_time: 0.5, body: "not".to_string() },
            WordTime { start_time: 0.6, end_time: 0.7, body: "is".to_string() },
            WordTime { start_time: 0.8, end_time: 0.9, body: "good".to_string() }
        ];
        let result = html_words("banana is good", Some(&timings));
        assert!(result.is_ok());
        let result_string = result.unwrap();
        let expected_string = "<span word='3' char='7' start='0.6' end='0.7' debug_body='is'>is</span> <span word='4' char='10' start='0.8' end='0.9' debug_body='good'>good</span>";
        assert_eq!(result_string, expected_string);
    }
}
