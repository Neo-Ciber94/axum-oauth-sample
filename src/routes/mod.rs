use self::auth_google::google_auth_router;
use crate::models::User;
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
    let session_cookie = cookies.get("auth_session");

    let Some(session_cookie) = session_cookie else {
        return Err(ErrorResponse::from(
            StatusCode::UNAUTHORIZED.into_response(),
        ));
    };

    let session_id = session_cookie.value();
    let user = sqlx::query_as!(
        User,
        r#"
            SELECT user.*
            FROM user
            LEFT JOIN user_session AS session ON session.user_id = user.id
            WHERE session.id = ?1
        "#,
        session_id
    )
    .fetch_optional(&pool)
    .await
    .unwrap();

    match user {
        Some(user) => Ok(Json(user).into_response()),
        None => Err(ErrorResponse::from(StatusCode::NOT_FOUND.into_response())),
    }
}

pub async fn logout(
    mut cookies: CookieJar,
    Extension(pool): Extension<SqlitePool>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let session_cookie = cookies.get("auth_session");

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

    let mut remove_session_cookie = Cookie::new("auth_session", "");
    remove_session_cookie.make_removal();
    cookies = cookies
        .add(remove_session_cookie)
        .remove(Cookie::from("auth_session"));
    Ok((cookies, Redirect::temporary("/")))
}
