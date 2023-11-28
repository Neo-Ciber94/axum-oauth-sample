use axum::{routing::get, Router};

use self::auth_google::google_auth_router;

mod auth_google;

pub fn auth_router() -> Router {
    Router::new()
        .nest("/auth", google_auth_router())
        .route("/auth/me", get(me))
}

pub async fn me() -> &'static str {
    "Auth me"
}
