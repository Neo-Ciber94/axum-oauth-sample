use self::auth_google::google_auth_router;
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
mod auth_google;

pub fn auth_router() -> Router {
    Router::new()
        .nest("/auth", google_auth_router())
        .route("/auth/me", get(me))
        .route("/auth/logout", get(logout))
}

pub async fn me(
    cookies: CookieJar,
    Extension(pool): Extension<SqlitePool>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let session_cookie = cookies.get(COOKIE_AUTH_SESSION);

    let Some(session_cookie) = session_cookie else {
        return Err(ErrorResponse::from(
            StatusCode::UNAUTHORIZED.into_response(),
        ));
    };

    let user = crate::db::get_user_by_session_id(&pool, session_cookie.value())
        .await
        .map_err(|_| ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR))?;

    match user {
        Some(user) => Ok(Json(user).into_response()),
        None => Err(ErrorResponse::from(StatusCode::NOT_FOUND.into_response())),
    }
}

pub async fn logout(
    mut cookies: CookieJar,
    Extension(pool): Extension<SqlitePool>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let session_cookie = cookies.get(COOKIE_AUTH_SESSION);

    let Some(session_cookie) = session_cookie else {
        return Err(ErrorResponse::from(
            StatusCode::UNAUTHORIZED.into_response(),
        ));
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
