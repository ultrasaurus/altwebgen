pub mod config;
pub mod devserve;
pub mod setup;

pub mod util;
pub mod web;
pub use web::words as words;

#[cfg(test)]
mod tests {

    use crate::{config::*, setup};
    use std::path::Path;
    // use test_log::test; // logging output for debugging tests

#[test]
// tests a sample app to verify that assets are copied into outdir when building
fn test_sample_template() {
    let config = Config::new("src/test/sample-template/.dist",
                                    "src/test/sample-template/source",
                                    "src/test/sample-template/template",
                                    "", Mode::Build, Transcript::Off);

    let result = setup::init_and_build(&config);
    if result.is_err() {
        println!("Err: {:?}", result.err());
         assert!(false, "unexpected failure of setup::init_and_build");
    }

    // test that static files were copied
    assert!(Path::new("src/test/sample-template/.dist/theme/theme.css").exists());
    assert!(Path::new("src/test/sample-template/.dist/theme/pixeltrue-web-design.png").exists());
    assert!(Path::new("src/test/sample-template/.dist/theme/icons8-favicon-pulsar-color.ico").exists());

}

fn audio_sample_config(transcript_opt: Transcript) -> Config{
    Config::new("src/test/sample-audio/.dist",
                                    "src/test/sample-audio/source",
                                    "src/test/sample-audio/template",
                                    "", Mode::Build, transcript_opt)
}

#[test]
fn test_sample_audio_no_transcript() {
    use kuchikiki::traits::TendrilSink;
    use crate::util::NodeRefExt;

    let config = audio_sample_config(Transcript::Off);

    let _ = setup::init_and_build(&config).unwrap();
    // test generated html
    let index_file_path = "src/test/sample-audio/.dist/index.html";
    assert!(Path::new(index_file_path).exists());
    let html = std::fs::read_to_string(index_file_path).unwrap();
    let document = kuchikiki::parse_html().one(html);
    // let stdout: std::io::Stdout = std::io::stdout();
    // let mut handle: std::io::StdoutLock<'_> = stdout.lock();
    // let _ = document.serialize(&mut handle);
    let audio_node = document.find_html_child_element("audio").unwrap();
    let maybe_paragraph = audio_node.as_node().next_sibling();
    assert!(maybe_paragraph.is_some());
    let paragraph = maybe_paragraph.unwrap();
    assert_eq!(paragraph.children().count(), 1);
    let text = paragraph.text_contents();
    assert_eq!(text.as_str(), "Let me introduce the word \"hypertext\" to mean a body of written or pictorial material interconnected in such a complex way that it could not conveniently be presented or represented on paper.");
}


}