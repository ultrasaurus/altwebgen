use axum::{
    routing::get,
    Router,
};
use tracing::{info};
use crate::web;
use tower_http::services::ServeDir;

// use std::borrow::Cow;

pub async fn run() {
    info!("devserve::run");
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(web::render_root))
        .nest_service("/image", ServeDir::new("source/image"))
        .nest_service("/style", ServeDir::new("source/style"))
        .route("/*path", get(web::render));


    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

