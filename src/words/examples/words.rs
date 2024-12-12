use words::html_words;

fn main() -> anyhow::Result<()> {
    let text = "2 much is never enough 4me... does punctuation work?";
    let data = html_words(text, None)?;

    println!("-----");
    println!("{}", data.html);

    println!("-----");

    Ok(())
}
