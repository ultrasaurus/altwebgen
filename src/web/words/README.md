# words-rs

Creates HTML spans for each word in a text string

If there is a file with same root name as the given inpute text file with extension ".timestamp.json" to provide wrd-level timestamp data, then each span is annotated with start/end time attribute(s).


TODO:
- figure out what attributes are needed
  - audio: tart time, duration
  - text: is word index useful? or just character index + word length?
- flag to turn on/off debug attribute
- use trace level logging and supress debug output by default

## Usage
install binary command-line tool locally:
```
cargo install --path .
```

Sample commands
```
words --help
words -i testdata/hypertext.txt
```

## Development

`cargo run` - uses default text "Hello world!"

expected output:
```
<span word='0' char='0'>Hello</span> <span word='1' char='6'>world</span>!
```