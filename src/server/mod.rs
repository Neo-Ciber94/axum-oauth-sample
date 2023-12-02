use askama_axum::IntoResponse;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::{async_trait, Extension};
use axum_extra::extract::CookieJar;
use sqlx::SqlitePool;

use crate::constants::{COOKIE_AUTH_SESSION, COOKIE_THEME};
use crate::misc::Theme;
use crate::models::User;

#[derive(Debug)]
pub struct CurrentUser(pub User);

pub struct UnauthorizedUser;
impl IntoResponse for UnauthorizedUser {
    fn into_response(self) -> askama_axum::Response {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = UnauthorizedUser;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(pool) = Extension::<SqlitePool>::from_request_parts(parts, state)
            .await
            .map_err(|err| {
                tracing::error!("{err}");
                UnauthorizedUser
            })?;
        let cookies = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|err| {
                tracing::error!("{err}");
                UnauthorizedUser
            })?;

        let Some(session_cookie) = cookies.get(COOKIE_AUTH_SESSION) else {
            return Err(UnauthorizedUser);
        };

        let user = crate::db::get_user_by_session_id(&pool, session_cookie.value())
            .await
            .map_err(|err| {
                tracing::error!("failed to get current user: {err}");
                UnauthorizedUser
            })?;

        match user {
            Some(x) => Ok(CurrentUser(x)),
            None => Err(UnauthorizedUser),
        }
    }
}

#[derive(Debug, Default)]
pub struct UserTheme(pub Option<Theme>);

#[async_trait]
impl<S> FromRequestParts<S> for UserTheme
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookies = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let Some(theme_cookie) = cookies.get(COOKIE_THEME) else {
            return Ok(Default::default());
        };

        match theme_cookie.value() {
            "dark" => Ok(UserTheme(Some(Theme::Dark))),
            "light" => Ok(UserTheme(Some(Theme::Light))),
            _ => Ok(Default::default()),
        }
    }
}
