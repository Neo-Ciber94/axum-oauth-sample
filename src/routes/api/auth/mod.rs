use self::{
    auth_discord::discord_auth_router, auth_github::github_auth_router,
    auth_google::google_auth_router,
};
use crate::constants::COOKIE_AUTH_SESSION;
use axum::{
    http::StatusCode,
    response::{ErrorResponse, IntoResponse, Redirect},
    routing::get,
    Extension, Json, Router,
};
use axum_extra::extract::cookie::CookieJar;
use cookie::Cookie;
use sqlx::SqlitePool;

mod auth_discord;
mod auth_github;
mod auth_google;

pub fn auth_router() -> Router {
    Router::new()
        .route("/api/auth/me", get(me))
        .route("/api/auth/logout", get(logout))
        .merge(google_auth_router())
        .merge(github_auth_router())
        .merge(discord_auth_router())
}

pub async fn me(
    cookies: CookieJar,
    Extension(pool): Extension<SqlitePool>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let session_cookie = cookies.get(COOKIE_AUTH_SESSION);

    let Some(session_cookie) = session_cookie else {
        return Err(ErrorResponse::from(StatusCode::UNAUTHORIZED));
    };

    let user = crate::db::get_user_by_session_id(&pool, session_cookie.value())
        .await
        .map_err(|_| ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR))?;

    match user {
        Some(user) => Ok(Json(user).into_response()),
        None => Err(ErrorResponse::from(StatusCode::NOT_FOUND)),
    }
}

pub async fn logout(
    mut cookies: CookieJar,
    Extension(pool): Extension<SqlitePool>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let session_cookie = cookies.get(COOKIE_AUTH_SESSION);

    let Some(session_cookie) = session_cookie else {
        return Err(ErrorResponse::from(StatusCode::UNAUTHORIZED));
    };

    crate::db::delete_user_session(&pool, session_cookie.value())
        .await
        .map_err(|_| ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR))?;

    let mut remove_session_cookie = Cookie::new(COOKIE_AUTH_SESSION, "");
    remove_session_cookie.set_path("/");
    remove_session_cookie.make_removal();

    cookies = cookies.add(remove_session_cookie);
    Ok((cookies, Redirect::to("/")))
}
