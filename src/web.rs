use anyhow::{Result, bail};
use std::fs::File;
use std::borrow::Cow;
use std::io::Read;
use tracing::info;
use axum::{response::Html, extract};

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

fn return_file_as_html(filepath: &str) -> Html<Cow<'static, str>> {
    let result: std::prelude::v1::Result<String, anyhow::Error> = read_file(filepath);
    match result {
        Ok(s) => Html(s.into()),
        Err(e) => {
            let error = format!("Error #{:?}", e);
            Html(error.into())
        }
    }
}

pub async fn render_root() -> Html<Cow<'static, str>> {
    info!("render_root");
    return_file_as_html("source/index.html")
}

pub async fn render(extract::Path(path): extract::Path<String>)
    -> Html<Cow<'static, str>> {
    info!("render path: #{}", path);
    return_file_as_html("source/index.html")

}