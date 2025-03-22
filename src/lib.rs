pub mod config;
pub mod devserve;
pub mod setup;

pub mod util;
pub mod watch;
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
                                    "", Mode::Build);

    let result = setup::init_and_build(&config);
    if result.is_err() {
        println!("Err: {:?}", result.err());
         assert!(false, "unexpected failure of setup::init_and_build");
    }

    // test that static files were copied
    assert!(Path::new("src/test/sample-template/.dist/theme/theme.css").exists());
    assert!(Path::new("src/test/sample-template/.dist/theme/pixeltrue-web-design.svg").exists());
    assert!(Path::new("src/test/sample-template/.dist/theme/icons8-favicon-pulsar-color.ico").exists());

}

}