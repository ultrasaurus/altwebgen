use words::html_words;

fn main() -> anyhow::Result<()> {
    let text = "2 much is never enough 4me... does punctuation work?";
    let (html_string, _word_index, _last_timing_index) = html_words(text, None)?;

    println!("-----");
    println!("{}", html_string);

    println!("-----");

    Ok(())
}
