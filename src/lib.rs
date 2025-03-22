pub mod config;
pub mod devserve;
pub mod setup;

pub mod util;
pub mod watch;
pub mod web;
pub use web::words as words;

#[cfg(test)]
mod tests {
    use crate::{config, setup};
    use std::path::Path;

#[test]
fn test_sample_template() {
    let config = config::Config::default();
    let _hbs = setup::init_and_build(&config).unwrap();

    // test that static files were copied
    assert!(Path::new("src/test/sample-template/.dist/theme/theme.css").exists());
    assert!(Path::new("src/test/sample-template/.dist/theme/pixeltrue-web-design.svg").exists());
    assert!(Path::new("src/test/sample-template/.dist/theme/icons8-favicon-pulsar-color.ico").exists());

}

}