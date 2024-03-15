use axum::{
    extract,
    Extension,
    http::StatusCode,
    middleware,
    response::Html,
    routing::get,
    Router,
};
use std::borrow::Cow;
use tracing::info;
use tower_http::services::ServeDir;
use crate::{config::Config, web};
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

fn return_file_as_html_response(filepath: &str) -> (StatusCode, Html<Cow<'static, str>>) {
    let result: std::prelude::v1::Result<String, anyhow::Error> = web::read_file_to_string(filepath);
    match result {
        Ok(s) => (StatusCode::OK, Html(s.into())),
        Err(e) => {
            let err_status = match e {
                _ => StatusCode::INTERNAL_SERVER_ERROR
            };
        (err_status, Html(format!("Error rendering {}<br>{:?}", filepath, e).into()))
        }
    }
}

pub async fn render_root(Extension(config): Extension<Config>) -> (StatusCode, Html<Cow<'static, str>>) {
    info!("render_root");
    // TODO: configure this, use SOURCE_ROOT or whatever it becomes
    return_file_as_html_response(format!("{}/index.html", config.outdir.display()).as_str())
}

pub async fn render(Extension(config): Extension<Config>,
                extract::Path(path): extract::Path<String>)
    -> (StatusCode, Html<Cow<'static, str>>) {
    let filepath = format!("{}/{}", config.outdir.display(), path);
    info!("render path: {}", path);
    return_file_as_html_response(&filepath)

}

