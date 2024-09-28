# words-rs

Creates HTML spans for each word in a text string

TODO:
- option to provide timestamp data and annotate each span with time attribute(s)
- figure out what attributes are needed
  - audio: tart time, duration
  - text: is word index useful? or just character index + word length?
- use trace level logging and supress debug output by default

## Usage
install binary command-line tool locally:
```
cargo install --path .
```


## Development

`cargo run` - uses default text "Hello world!"

expected output:
```
<span word='0' char='0'>Hello</span> <span word='1' char='6'>world</span>!
```