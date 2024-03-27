// experiments learning how to use regex
use regex::Regex;

fn main() -> anyhow::Result<()> {
    let text = "Hello, everyone in the world!";
    println!("text: {}", text);

    println!("-- captures_iter ---");
//    let regex: Regex = Regex::new(r"([a-zà-ýA-ZÀ-Ý0-9]+?)[[\s$][^a-zà-ýA-ZÀ-Ý0-9]]+?")?;
    let regex = Regex::new(r"([a-zà-ýA-ZÀ-Ý0-9]+?)([[\s$][^a-zà-ýA-ZÀ-Ý0-9]]+)")?;
    let mut nth_word = 0;
    let html_string = regex.captures_iter(text).map(|c| {
        println!("{:?}", c);
        let range: std::ops::Range<usize> = c.get(0).unwrap().range();
        let s = format!("<span word='{}' char='{}'>{}</span>{}", 
                        nth_word, range.start, &c[1], &c[2]);
        nth_word = nth_word + 1;
        s
    }).collect::<Vec<String>>().join("");



    println!("-----");
    println!("{}", html_string);

    println!("-----");



    Ok(())
}
