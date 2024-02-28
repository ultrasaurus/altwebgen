use axum::{
    extract,
    http::StatusCode,
    response::Html,
    routing::get,
    Router,
};
use std::borrow::Cow;
use tracing::{info};
use crate::web;
use tower_http::services::ServeDir;

// use std::borrow::Cow;

pub async fn run() {
    info!("devserve::run");
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(render_root))
        .nest_service("/image", ServeDir::new("source/image"))
        .nest_service("/style", ServeDir::new("source/style"))
        .route("/*path", get(render));


    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
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
        (err_status, Html(format!("Error #{:?}", e).into()))
        }
    }
}

const SOURCE_ROOT: &str = "source/";    // TODO: config 
pub async fn render_root() -> (StatusCode, Html<Cow<'static, str>>) {
    info!("render_root");
    // TODO: configure this, use SOURCE_ROOT or whatever it becomes
    return_file_as_html_response("source/index.html")
}

pub async fn render(extract::Path(path): extract::Path<String>)
    -> (StatusCode, Html<Cow<'static, str>>) {
    let filepath = format!("{}{}", SOURCE_ROOT, path);
    info!("render path: {}", path);
    return_file_as_html_response(&filepath)

}