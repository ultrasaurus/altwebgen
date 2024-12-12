#![allow(dead_code, unused)]    // TODO: remove after implementation
use serde::Deserialize;
use std::io::Read;
use anyhow::anyhow;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordTime {
    pub start_time: f32,
    pub end_time: f32,
    pub body: String
}

#[derive(Debug, Deserialize)]
struct Transcript {
    version: String,
    segments: Vec<WordTime>,
}

impl WordTime {
    pub fn from_transcript<R: Read>(reader: R) -> anyhow::Result<Vec<WordTime>> {
        // let Transcript{version, segments: words} = Transcript {
        //     version: String::from("1.0.0"), 
        //     segments: Vec::new()
        // };
         match serde_json::from_reader(reader) {
            Ok(Transcript{version, segments: words}) => {
                if version != "1.0.0" {
                    println!("version {:?}", version); // TODO: use warn log 
                }
                Ok(words)
            },
            Err(e) => {
                println!("err {:?}", e); // TODO: use err log 
                Err(anyhow!(e).context("from_transcript: failed to convert to json"))
            }

        }

    }

}

#[cfg(test)]
mod tests {
    use super::WordTime;

    #[test]
    fn from_transcript_empty_string() {
        let empty = "".as_bytes();
        let result = WordTime::from_transcript(empty); 
        assert!(result.is_err());
    }

    fn from_transcript_empty_json_object() {
        let empty = "{}".as_bytes();
        let result = WordTime::from_transcript(empty); 
        assert!(result.is_ok());
        if let Ok(vec) = result {
            assert_eq!(vec.len(), 0);
        }
    }


    #[test]
    fn from_transcript_one_word() {
        let transcript = r##"
        {
            "version": "1.0.0",
            "segments": [
                {
                    "startTime": 0,
                    "endTime": 0.2399999999999931,
                    "body": "let"
                }
            ]
    }
        "##.as_bytes();
        let result = WordTime::from_transcript(transcript); 
        assert!(result.is_ok());
        if let Ok(vec) = result {
            assert_eq!(vec.len(), 1);
            assert_eq!(vec[0].body, String::from("let"));
            assert_eq!(vec[0].start_time, 0.0);
            assert_eq!(vec[0].end_time, 0.2399999999999931);

        }
    }


}