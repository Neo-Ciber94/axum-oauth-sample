mod constants;
mod models;
mod routes;

use axum::{routing::get, Extension, Router};
use dotenvy::dotenv;
use sqlx::sqlite::SqlitePool;
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

    let connection_string = std::env::var("DATABASE_URL")?;
    let pool = SqlitePool::connect(&connection_string).await?;

    let app = Router::new()
        .route("/", get(hello))
        .merge(auth_router())
        .layer(Extension(pool))
        .layer(TraceLayer::new_for_http());

    let (host, port) = get_host_and_port()?;
    let listener = tokio::net::TcpListener::bind((host.as_str(), port)).await?;
    tokio::net::TcpListener::bind((host.as_str(), port)).await?;
    println!("Listening on: http://{host}:{port}");
    axum::serve(listener, app).await?;

    Ok(())
}

fn get_host_and_port() -> Result<(String, u16), Box<dyn Error>> {
    let host = std::env::var("HOST").map_err(|_| format!("PORT not found"))?;
    let port = std::env::var("PORT")
        .map_err(|_| format!("PORT not found"))
        .and_then(|x| match u16::from_str_radix(&x, 10) {
            Ok(port) => Ok(port),
            Err(_) => Err(format!("Invalid port: {x}")),
        })?;

    Ok((host, port))
}

async fn hello() -> &'static str {
    return "Hello World!";
}
