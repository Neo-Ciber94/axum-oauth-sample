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
        _ => Err(Redirect::temporary("/login")),
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
        Some(_) => Err(Redirect::temporary("/")),
        None => Ok(LoginTemplate { theme, user: None }),
    }
}
