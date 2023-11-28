mod routes;

use axum::{routing::get, Router};
use dotenv::dotenv;
use std::error::Error;
use tower_http::trace::TraceLayer;
use tracing::Level;

use crate::routes::auth_router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let host = std::env::var("HOST").map_err(|_| format!("PORT not found"))?;
    let port = std::env::var("PORT")
        .map_err(|_| format!("PORT not found"))
        .and_then(|x| match u16::from_str_radix(&x, 10) {
            Ok(port) => Ok(port),
            Err(_) => Err(format!("Invalid port: {x}")),
        })?;

    let app = Router::new()
        .route("/", get(hello))
        .merge(auth_router())
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind((host.as_str(), port)).await?;
    println!("Listening on: http://{host}:{port}");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn hello() -> &'static str {
    return "Hello World!";
}
