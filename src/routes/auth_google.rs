use axum::{
    routing::{get, post},
    Router,
};

pub fn google_auth_router() -> Router {
    Router::new()
        .route("/google/login", get(login))
        .route("/google/callback", post(callback))
}

async fn login() -> &'static str {
    return "Google Login";
}

async fn callback() -> &'static str {
    return "Google callback";
}
