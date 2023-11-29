use self::auth_google::google_auth_router;
use crate::{constants::COOKIE_AUTH_SESSION, models::User};
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

    let session_id = session_cookie.value();
    let user = sqlx::query_as!(
        User,
        r#"
            SELECT user.id as "id: _", account_id, username
            FROM user
            LEFT JOIN user_session AS session ON session.user_id = user.id
            WHERE session.id = ?1
        "#,
        session_id
    )
    .fetch_optional(&pool)
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

    let session_id = session_cookie.value();

    let mut conn = pool
        .acquire()
        .await
        .map_err(|_| ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR))?;

    sqlx::query!("DELETE FROM user_session WHERE id = ?1", session_id)
        .execute(&mut *conn)
        .await
        .map_err(|_| ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR))?;

    let mut remove_session_cookie = Cookie::new(COOKIE_AUTH_SESSION, "");
    remove_session_cookie.set_path("/");
    remove_session_cookie.make_removal();

    cookies = cookies.add(remove_session_cookie);
    Ok((cookies, Redirect::temporary("/")))
}
