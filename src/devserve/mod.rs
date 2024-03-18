use std::path::{Path, PathBuf};
use tracing::{info, warn};
use warp::Filter;
use crate::config::Config;
mod watch;
use watch::watch;

pub async fn serve<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
    info!("serve website_dir: {}", path.as_ref().display());
    let website_dir = PathBuf::from(path.as_ref());
    let index_path = website_dir.join("index.html");
     // GET / => index.html
    let root = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file(index_path));

    // dir already requires GET...
    let all = warp::fs::dir(website_dir);

    let routes = root.or(all)
                .with(warp::trace::request());

    warp::serve(routes).run(([127, 0, 0, 1], 3456)).await;
    Ok(())
}


pub async fn run(config: &Config) -> anyhow::Result<()> {
    info!("devserve::run");
    let website_dir = config.outdir.canonicalize()?;

    loop {
        tokio::select! {
            files = watch(&website_dir) => { info!("watch: files changed: {:?}", files)},
            _ = serve(&website_dir) => { info!("serving done")},
        }
        info!("done!!");
    }


    Ok(())
}