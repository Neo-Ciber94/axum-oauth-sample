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
        .fallback(not_found)
        .route(
            "/error",
            get(|| async { StatusCode::UNAUTHORIZED.into_response() }),
        )
        .layer(middleware::from_fn(handle_error))
}

#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate {
    theme: Theme,
    user: Option<User>,
}

async fn home(
    user: Option<CurrentUser>,
    UserTheme(theme): UserTheme,
) -> Result<HomeTemplate, Redirect> {
    let theme = theme.unwrap_or_default();

    match user {
        Some(CurrentUser(user)) => Ok(HomeTemplate {
            theme,
            user: Some(user),
        }),
        _ => Err(Redirect::to("/login")),
    }
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    theme: Theme,
    user: Option<User>,
}

async fn login(
    user: Option<CurrentUser>,
    UserTheme(theme): UserTheme,
) -> Result<LoginTemplate, Redirect> {
    let theme = theme.unwrap_or_default();

    match user {
        Some(_) => Err(Redirect::to("/")),
        None => Ok(LoginTemplate { theme, user: None }),
    }
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

async fn handle_error(
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

mod filters {
    pub fn take<T: std::fmt::Display>(s: T, count: usize) -> ::askama::Result<String> {
        let s = s.to_string();
        Ok(s[0..count].to_string())
    }
}
