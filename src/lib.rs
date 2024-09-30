use regex::Regex;
mod transcript;

pub fn html_words<S: AsRef<str>>(text: S) -> anyhow::Result<String> {
    let regex = Regex::new(r"([a-zà-ýA-ZÀ-Ý0-9]+?)([[\s$][^a-zà-ýA-ZÀ-Ý0-9]]+)")?;
    let mut nth_word = 0;
    let html_string = regex.captures_iter(text.as_ref()).map(|c| {
        println!("{:?}", c);
        let range: std::ops::Range<usize> = c.get(0).unwrap().range();
        let s = format!("<span word='{}' char='{}'>{}</span>{}", 
                        nth_word, range.start, &c[1], &c[2]);
        nth_word = nth_word + 1;
        s
    }).collect::<Vec<String>>().join("");
    Ok(html_string)    
}

#[cfg(test)]
mod tests {
    use super::*;

     #[test]
    fn html_words_empty_string() {
        let result = html_words("");
        assert!(result.is_ok());
        let result_string = result.unwrap();
        assert_eq!(result_string, "");
    }

    #[test]
    fn html_words_hello_world() {
        let result = html_words("Hello world!");
        assert!(result.is_ok());
        let result_string = result.unwrap();
        let expected_string = "<span word='0' char='0'>Hello</span> <span word='1' char='6'>world</span>!";
        assert_eq!(result_string, expected_string);
    }

}