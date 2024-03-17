use tracing::{error, info};
use warp::Filter;
use crate::config::Config;


// use std::borrow::Cow;

pub async fn run(config: &Config) -> anyhow::Result<()> {
    info!("devserve::run");
    let website_dir = config.outdir.canonicalize()?;
    info!("serve website_dir: {}", website_dir.display());
    let root = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file(website_dir.join("index.html")));

    // dir already requires GET...
    let all = warp::fs::dir(website_dir);

    // GET / => index.html
    // GET /... => ./examples/..
    let routes = root.or(all)
                .with(warp::trace::request());

    warp::serve(routes).run(([127, 0, 0, 1], 3456)).await;

    Ok(())
}