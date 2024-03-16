use axum::{
    body::Body,
    extract,
    Extension,
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio_util::io::ReaderStream;

use tracing::{error, info};
use tower_http::services::ServeDir;
use crate::config::Config;
mod log;
use log::*;

// use std::borrow::Cow;

pub async fn run(config: &Config) {
    info!("devserve::run");
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(render_root))
        .nest_service("/image", ServeDir::new("source/image"))
        .nest_service("/style", ServeDir::new("source/style"))
        .route("/*path", get(render))
        .layer(Extension(config.clone()))
        .layer(middleware::from_fn(print_request_response));



    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("localhost:3456").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_filestream<P: AsRef<std::path::Path>>(filepath: P) ->
    anyhow::Result<ReaderStream<tokio::fs::File>> {
    let path = std::fs::canonicalize(&filepath)?;
    info!("opening file #{}", &path.display());
    let f= tokio::fs::File::open(&path).await?;
    // convert the `AsyncRead` into a `Stream`
    Ok(ReaderStream::new(f))
}

async fn return_file_as_response <P: AsRef<std::path::Path>>(filepath: P)
    -> (StatusCode, Body) {
    let stream = match get_filestream(&filepath).await {
        Err(e) => {
            error!("Error opening file: {} -- {:?}", &filepath.as_ref().display(), e);
            return (StatusCode::NOT_FOUND, Body::from(format!("File not found: {}", e)))
        },
        Ok(s) => s,
    };
    let body = Body::from_stream(stream);
    // TODO: mime type
    (StatusCode::OK, body)
}

pub async fn render_root(Extension(config): Extension<Config>) -> impl IntoResponse {
    info!("render_root");
    return_file_as_response(config.outdir.join("index.html")).await
}

pub async fn render(Extension(config): Extension<Config>,
                extract::Path(path): extract::Path<String>)
    -> impl IntoResponse {
    info!("render path: {}", path);
    let filepath = config.outdir.join(path);
    return_file_as_response(&filepath).await

}

