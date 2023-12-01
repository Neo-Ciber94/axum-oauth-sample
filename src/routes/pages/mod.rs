use crate::{
    misc::Theme,
    models::User,
    server::{CurrentUser, UserTheme},
};
use askama::Template;
use axum::{response::Redirect, routing::get, Router};

pub fn pages_router() -> Router {
    Router::new()
        .route("/", get(home))
        .route("/login", get(login))
        .fallback(not_found)
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
#[template(path = "404.html")]
struct NotFoundTemplate {
    theme: Theme,
    user: Option<User>,
}

async fn not_found(UserTheme(theme): UserTheme) -> NotFoundTemplate {
    let theme = theme.unwrap_or_default();
    NotFoundTemplate { theme, user: None }
}

mod filters {
    pub fn take<T: std::fmt::Display>(s: T, count: usize) -> ::askama::Result<String> {
        let s = s.to_string();
        Ok(s[0..count].to_string())
    }
}
