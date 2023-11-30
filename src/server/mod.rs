use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::{async_trait, Extension};
use axum_extra::extract::CookieJar;
use sqlx::SqlitePool;

use crate::constants::COOKIE_AUTH_SESSION;
use crate::models::User;

#[derive(Debug)]
pub struct CurrentUser(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(pool) = Extension::<SqlitePool>::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let cookies = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let Some(session_cookie) = cookies.get(COOKIE_AUTH_SESSION) else {
            return Err(StatusCode::UNAUTHORIZED);
        };

        let user = crate::db::get_user_by_session_id(&pool, session_cookie.value())
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        match user {
            Some(x) => Ok(CurrentUser(x)),
            None => Err(StatusCode::UNAUTHORIZED),
        }
    }
}
