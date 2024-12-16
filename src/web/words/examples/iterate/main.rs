use words::*;

const TEXT:&str = r#"
Sonnet 43
When most I wink, then do mine eyes best see,
For all the day they view things unrespected;
But when I sleep, in dreams they look on thee,
And darkly bright, are bright in dark directed.
Then thou, whose shadow shadows doth make bright,
How would thy shadow’s form form happy show
To the clear day with thy much clearer light,
When to unseeing eyes thy shade shines so!
How would, I say, mine eyes be blessed made
By looking on thee in the living day,
When in dead night thy fair imperfect shade
Through heavy sleep on sightless eyes doth stay!
    All days are nights to see till I see thee,
    And nights bright days when dreams do show thee me.
"#;

fn main(){

    TEXT.lines().for_each( |line| {
        match html_words(line, None) {
            Ok(HtmlWords{html, word_index:_, last_timing_index:_}) => println!("<p>{}</p>", html),
            Err(e) => println!("***error with line {}\n   {:?}", line, e)
        }
    })
}