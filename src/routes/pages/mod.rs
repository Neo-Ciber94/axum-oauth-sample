use crate::{
    misc::{PageError, Theme},
    models::User,
    server::{CurrentUser, UserTheme},
};
use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    extract::Request, http::StatusCode, middleware, middleware::Next, response::Redirect,
    routing::get, Router,
};

pub fn pages_router() -> Router {
    Router::new()
        .route("/", get(home))
        .route("/login", get(login))
        .layer(middleware::from_fn(auth_middleware))
        .fallback(not_found)
}

#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate {
    theme: Theme,
    user: Option<User>,
}

async fn home(CurrentUser(user): CurrentUser, UserTheme(theme): UserTheme) -> HomeTemplate {
    let theme = theme.unwrap_or_default();
    HomeTemplate {
        theme,
        user: Some(user),
    }
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    theme: Theme,
    user: Option<User>,
}

async fn login(UserTheme(theme): UserTheme) -> LoginTemplate {
    let theme = theme.unwrap_or_default();
    LoginTemplate { theme, user: None }
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate {
    theme: Theme,
    user: Option<User>,
    error: PageError,
}

async fn not_found(UserTheme(theme): UserTheme) -> ErrorTemplate {
    let theme = theme.unwrap_or_default();

    ErrorTemplate {
        theme,
        user: None,
        error: PageError {
            message: "Not Found".to_owned(),
            status: StatusCode::NOT_FOUND,
        },
    }
}

pub async fn error_handler_middleware(
    UserTheme(theme): UserTheme,
    request: Request,
    next: Next,
) -> axum::response::Response {
    let response = next.run(request).await;

    if response.status().is_client_error() || response.status().is_server_error() {
        let theme = theme.unwrap_or_default();
        let status = response.status();
        let message = status
            .canonical_reason()
            .map(|s| s.to_owned())
            .unwrap_or_else(|| "Something went wrong".to_owned());

        return ErrorTemplate {
            theme,
            user: None,
            error: PageError { status, message },
        }
        .into_response();
    }

    response
}

async fn auth_middleware(
    user: Option<CurrentUser>,
    request: Request,
    next: Next,
) -> axum::response::Response {
    match user {
        None if request.uri().path() != "/login" => Redirect::to("/login").into_response(),
        Some(_) if request.uri().path() == "/login" => Redirect::to("/").into_response(),
        _ => next.run(request).await,
    }
}

mod filters {
    pub fn take<T: std::fmt::Display>(s: T, count: usize) -> ::askama::Result<String> {
        let s = s.to_string();
        Ok(s[0..count].to_string())
    }
}
