use words::html_words;

fn main() -> anyhow::Result<()> {
    let text = "2 much is never enough 4me... does punctuation work?";
    let html_string = html_words(text, None)?;

    println!("-----");
    println!("{}", html_string);

    println!("-----");

    Ok(())
}
