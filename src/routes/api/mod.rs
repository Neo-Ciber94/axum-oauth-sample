mod auth;

use askama_axum::IntoResponse;
use axum::{
    http::{header, HeaderMap},
    response::Redirect,
    routing::post,
    Router,
};
use axum_extra::extract::CookieJar;
use cookie::Cookie;

use crate::{constants::COOKIE_THEME, misc::Theme, server::UserTheme};

pub fn api_router() -> Router {
    Router::new()
        .merge(auth::auth_router())
        .route("/api/toggle_theme", post(toggle_theme))
}

async fn toggle_theme(UserTheme(theme): UserTheme, headers: HeaderMap) -> impl IntoResponse {
    let theme = theme.unwrap_or_default();

    let new_theme = match theme {
        Theme::Dark => Theme::Light,
        Theme::Light => Theme::Dark,
    };

    let theme_cookie: Cookie = Cookie::build((COOKIE_THEME, new_theme.to_string()))
        .path("/")
        .permanent()
        .into();

    let cookies = CookieJar::new().add(theme_cookie);
    let referer = headers.get(header::REFERER);
    let path = match referer {
        Some(referer) => referer.to_str().expect("Failed to get referer path"),
        None => "/",
    };

    (cookies, Redirect::to(path))
}
