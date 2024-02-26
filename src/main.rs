use axum::{
    routing::get,
    Router,
};
use tracing::{info};
mod web;
use web::*;
// use std::borrow::Cow;

#[tokio::main]
async fn main() {
    // install global subscriber configured based on RUST_LOG envvar.
    tracing_subscriber::fmt::init();
    info!("Logging enabled");
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(render_root))
        .route("/*path", get(render));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
// async fn root() -> String {
//     info!("root");
//     render().into()
// }

