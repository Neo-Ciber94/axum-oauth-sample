mod constants;
mod db;
mod misc;
mod models;
mod routes;
mod server;

use anyhow::Context;
use axum::{middleware, Extension, Router};
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

    // Database
    let connection_string = std::env::var("DATABASE_URL").context("'DATABASE_URL' no found")?;
    let pool = SqlitePool::connect(&connection_string)
        .await
        .context("Failed to connect to database")?;

    // Routes
    let app = Router::new()
        .merge(public_dir())
        .merge(crate::routes::api_router())
        .merge(crate::routes::pages_router())
        .layer(Extension(pool))
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(crate::routes::error_handler_middleware));

    // Start server
    let host = std::env::var("HOST").context("'HOST' no found")?;
    let port = std::env::var("PORT")
        .context("'PORT' no found")
        .and_then(|x| {
            x.parse::<u16>()
                .map_err(|_| anyhow::anyhow!("Invalid port: {x}"))
        })?;

    let listener = tokio::net::TcpListener::bind((host.as_str(), port))
        .await
        .context("Failed to start tcp listener")?;

    println!("Listening on: http://{host}:{port}");
    axum::serve(listener, app)
        .await
        .context("Failed to start server")?;

    Ok(())
}

fn public_dir() -> Router {
    Router::new().nest_service("/public", ServeDir::new("public"))
}
