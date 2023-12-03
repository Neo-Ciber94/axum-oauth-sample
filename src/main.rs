mod constants;
mod db;
mod misc;
mod models;
mod routes;
mod server;

use anyhow::Context;
use axum::{Extension, Router};
use dotenvy::dotenv;
use sqlx::sqlite::SqlitePool;
use std::error::Error;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let connection_string = std::env::var("DATABASE_URL").context("'DATABASE_URL' no found")?;
    let pool = SqlitePool::connect(&connection_string)
        .await
        .context("Failed to connect to database")?;

    let app = Router::new()
        .merge(crate::routes::api_router())
        .merge(crate::routes::pages_router())
        .merge(public_dir())
        .layer(Extension(pool))
        .layer(TraceLayer::new_for_http());

    let (host, port) = get_host_and_port()?;
    let listener = tokio::net::TcpListener::bind((host.as_str(), port))
        .await
        .context("Failed to start tcp listener")?;

    println!("Listening on: http://{host}:{port}");
    axum::serve(listener, app)
        .await
        .context("Failed to start server")?;

    Ok(())
}

fn get_host_and_port() -> Result<(String, u16), Box<dyn Error>> {
    let host = std::env::var("HOST").context("'HOST' no found")?;
    let port = std::env::var("PORT")
        .context("'PORT' no found")
        .and_then(|x| match x.parse::<u16>() {
            Ok(port) => Ok(port),
            Err(_) => Err(anyhow::anyhow!("Invalid port: {x}")),
        })?;

    Ok((host, port))
}

fn public_dir() -> Router {
    Router::new().nest_service("/public", ServeDir::new("public"))
}
