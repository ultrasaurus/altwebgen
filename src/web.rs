use anyhow::{Result, bail};
use std::fs::File;
use std::borrow::Cow;
use std::io::Read;
use tracing::info;
use axum::response::Html;

fn read_file(filename: &str) -> Result<String> {
    info!("read_file #{}", filename);
    let mut f = File::open(filename)?;
    let mut buf = String::new();
    let bytes = f.read_to_string(&mut buf)?;
    if bytes == 0 {
      bail!("failed to read: 0 bytes returned from read_to_string");  
    } 
    Ok(buf)
}

pub fn render() -> Html<Cow<'static, str>> {   
    let result = read_file("source/index.html");
    match result {
        Ok(s) => Html(s.into()),
        Err(e) => {
            let error = format!("Error #{:?}", e);
            Html(error.into())
        }
    }
}