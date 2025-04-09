use regex::Regex;
mod transcript;
pub use transcript::WordTime as WordTime;
use anyhow::Result;

#[derive(Debug)]
pub struct HtmlWords {
   pub html: String,
   pub word_index: usize,
   pub last_timing_index: usize
}

// Returns HtmlWords struct
//   html:  as a trimmed string,
//   word_index: the number of words (index of next word),
//   last_timing_index: and the last timing index used
pub fn html_words(text: &str, optional_timing: Option<&[WordTime]>) -> Result<HtmlWords> {
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
            print!(
                "Comparing: timing body '{}' with word '{}'",
                timing.body.to_lowercase(),
                word.to_lowercase()
            );

            if timing.body.to_lowercase() == word.to_lowercase() {
                // Match found
                println!(" -- match");
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
            } else {
                println!(" -- NO MATCH");
            }
        }

        // If no match was found, add error span
        if !matched {
            html_string.push_str(&format!(
                "<span word='{}' error='NO_MATCH' debug_body='{}'>{}</span> ",
                word_index,
                timings[last_timing_index].body,
                word
            ));
        }

        word_index += 1;
    }

    Ok(HtmlWords {
        html: html_string.trim().to_string(),
        word_index,
        last_timing_index
    })
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
        let data = result.unwrap();
        let expected_string = "";
        assert_eq!(data.html, expected_string);
        assert_eq!(data.word_index, 0);
        assert_eq!(data.last_timing_index, 0);
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
        let data = result.unwrap();
        let expected_string = "<span word='0' start='0' end='0.1' debug_body='hello'>Hello</span> <span word='1' start='0.2' end='0.3' debug_body='world'>world</span>";
        assert_eq!(data.html, expected_string);
        assert_eq!(data.word_index, 2);
        assert_eq!(data.last_timing_index, 2);
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
        let data= result.unwrap();
        let expected_string = "<span word='0' start='0' end='0.1' debug_body='hello'>Hello</span> <span word='1' error='NO_MATCH' debug_body='world'>there</span> <span word='2' start='0.2' end='0.3' debug_body='world'>world</span>";
        assert_eq!(data.html, expected_string);
        assert_eq!(data.word_index, 3);
        assert_eq!(data.last_timing_index, 2);
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
        let data = result.unwrap();
        let expected_string = "<span word='0' start='0' end='0.1' debug_body='hello'>Hello</span> <span word='1' error='NO_MATCH' debug_body='there'>my</span> <span word='2' start='0.4' end='0.5' debug_body='world'>world</span>";
        assert_eq!(data.html, expected_string);
        assert_eq!(data.word_index, 3);
        assert_eq!(data.last_timing_index, 3);
    }

    const SENTENCE_JSON: &str = r#"
[
        {
            "startTime": 1.634999999999991,
            "endTime": 1.7850000000000819,
            "body": "The"
        },
        {
            "startTime": 1.7850000000000819,
            "endTime": 2.0850000000000364,
            "body": "real"
        },
        {
            "startTime": 2.0850000000000364,
            "endTime": 2.4750000000001364,
            "body": "heart"
        },
        {
            "startTime": 2.4750000000001364,
            "endTime": 2.5650000000000546,
            "body": "of"
        },
        {
            "startTime": 2.5650000000000546,
            "endTime": 2.6549999999999727,
            "body": "the"
        },
        {
            "startTime": 2.6549999999999727,
            "endTime": 3.0150000000001,
            "body": "matter"
        },
        {
            "startTime": 3.0150000000001,
            "endTime": 3.105000000000018,
            "body": "of"
        },
        {
            "startTime": 3.105000000000018,
            "endTime": 3.7650000000001,
            "body": "selection,"
        },
        {
            "startTime": 3.7650000000001,
            "endTime": 4.4249999999999545,
            "body": "however"
        },
        {
            "startTime": 4.605000000000018,
            "endTime": 4.875,
            "body": "goes"
        },
        {
            "startTime": 4.875,
            "endTime": 5.205000000000155,
            "body": "deeper"
        },
        {
            "startTime": 5.205000000000155,
            "endTime": 5.355000000000018,
            "body": "than"
        },
        {
            "startTime": 5.355000000000018,
            "endTime": 5.445000000000164,
            "body": "a"
        },
        {
            "startTime": 5.445000000000164,
            "endTime": 5.9249999999999545,
            "body": "lag"
        },
        {
            "startTime": 5.9249999999999545,
            "endTime": 6.0750000000000455,
            "body": "in"
        },
        {
            "startTime": 6.0750000000000455,
            "endTime": 6.644999999999982,
            "body": "adoption"
        },
        {
            "startTime": 6.644999999999982,
            "endTime": 6.735000000000127,
            "body": "of"
        },
        {
            "startTime": 6.735000000000127,
            "endTime": 7.455000000000155,
            "body": "mechanisms"
        },
        {
            "startTime": 7.455000000000155,
            "endTime": 7.605000000000018,
            "body": "by"
        },
        {
            "startTime": 7.605000000000018,
            "endTime": 8.475000000000136,
            "body": "libraries"
        },
        {
            "startTime": 8.654999999999973,
            "endTime": 8.835000000000036,
            "body": "or"
        },
        {
            "startTime": 8.835000000000036,
            "endTime": 8.924999999999955,
            "body": "a"
        },
        {
            "startTime": 8.924999999999955,
            "endTime": 9.225000000000136,
            "body": "lack"
        },
        {
            "startTime": 9.225000000000136,
            "endTime": 9.345000000000027,
            "body": "of"
        },
        {
            "startTime": 9.345000000000027,
            "endTime": 9.884999999999991,
            "body": "development"
        },
        {
            "startTime": 9.884999999999991,
            "endTime": 9.975000000000136,
            "body": "of"
        },
        {
            "startTime": 9.975000000000136,
            "endTime": 10.634999999999991,
            "body": "devices"
        },
        {
            "startTime": 10.634999999999991,
            "endTime": 10.845000000000027,
            "body": "for"
        },
        {
            "startTime": 10.845000000000027,
            "endTime": 11.025000000000091,
            "body": "their"
        },
        {
            "startTime": 11.025000000000091,
            "endTime": 11.445000000000164,
            "body": "use."
        }
    ]
    "#;
    #[test]
    fn html_words_mismatch_multiples_in_sentence() {
        let timings: Vec<WordTime> = serde_json::from_str(SENTENCE_JSON).unwrap();
        println!("timintimings.len() = {}", timings.len());
        let text = "The real heart of the matter of selection, however, goes deeper than a lag in the adoption of mechanisms by libraries, or a lack of development of devices for their use.";
        print!("num words = {}", text.split(" ").count());
        let result = html_words(text, Some(&timings));
        assert!(result.is_ok());
        let data = result.unwrap();
        let expected_string = "<span word='0' start='1.635' end='1.785' debug_body='The'>The</span> <span word='1' start='1.785' end='2.085' debug_body='real'>real</span> <span word='2' start='2.085' end='2.475' debug_body='heart'>heart</span> <span word='3' start='2.475' end='2.565' debug_body='of'>of</span> <span word='4' start='2.565' end='2.655' debug_body='the'>the</span> <span word='5' start='2.655' end='3.015' debug_body='matter'>matter</span> <span word='6' start='3.015' end='3.105' debug_body='of'>of</span> <span word='7' error='NO_MATCH' debug_body='selection,'>selection</span> <span word='8' start='3.765' end='4.425' debug_body='however'>however</span> <span word='9' start='4.605' end='4.875' debug_body='goes'>goes</span> <span word='10' start='4.875' end='5.205' debug_body='deeper'>deeper</span> <span word='11' start='5.205' end='5.355' debug_body='than'>than</span> <span word='12' start='5.355' end='5.445' debug_body='a'>a</span> <span word='13' start='5.445' end='5.925' debug_body='lag'>lag</span> <span word='14' start='5.925' end='6.075' debug_body='in'>in</span> <span word='15' error='NO_MATCH' debug_body='adoption'>the</span> <span word='16' start='6.075' end='6.645' debug_body='adoption'>adoption</span> <span word='17' start='6.645' end='6.735' debug_body='of'>of</span> <span word='18' start='6.735' end='7.455' debug_body='mechanisms'>mechanisms</span> <span word='19' start='7.455' end='7.605' debug_body='by'>by</span> <span word='20' start='7.605' end='8.475' debug_body='libraries'>libraries</span> <span word='21' start='8.655' end='8.835' debug_body='or'>or</span> <span word='22' start='8.835' end='8.925' debug_body='a'>a</span> <span word='23' start='8.925' end='9.225' debug_body='lack'>lack</span> <span word='24' start='9.225' end='9.345' debug_body='of'>of</span> <span word='25' start='9.345' end='9.885' debug_body='development'>development</span> <span word='26' start='9.885' end='9.975' debug_body='of'>of</span> <span word='27' start='9.975' end='10.635' debug_body='devices'>devices</span> <span word='28' start='10.635' end='10.845' debug_body='for'>for</span> <span word='29' start='10.845' end='11.025' debug_body='their'>their</span> <span word='30' error='NO_MATCH' debug_body='use.'>use</span>";
        assert_eq!(data.html, expected_string);
        assert_eq!(data.word_index, 31);
        assert_eq!(data.last_timing_index, 29); // will be timings.len() when punctuation is ignored
    }


}