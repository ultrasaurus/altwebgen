use altwebgen::words::*;

fn main() -> anyhow::Result<()> {
    let text: &str = "Let me introduce";
    let timing_json = r#"{
    "version": "1.0.0",
    "segments": [
        {
            "startTime": 0,
            "endTime": 0.2399999999999931,
            "body": "let"
        },
        {
            "startTime": 0.2399999999999931,
            "endTime": 0.3599999999999923,
            "body": "me"
        },
        {
            "startTime": 0.3599999999999923,
            "endTime": 0.8699999999999921,
            "body": "introduce"
        }
    ]}
    "#.as_bytes();
    let timings = WordTime::from_transcript(timing_json)?;

    let data = html_words(text, Some(&timings))?;

    println!("-----");
    println!("{}", data.html);

    println!("-----");

    Ok(())
}
