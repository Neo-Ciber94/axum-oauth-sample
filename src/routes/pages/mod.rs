use std::fmt::Display;

use askama::Template;
use axum::{response::Redirect, routing::get, Router};

use crate::{models::User, server::CurrentUser};

pub fn pages_router() -> Router {
    Router::new()
        .route("/", get(home))
        .route("/login", get(login))
}

enum Theme {
    Dark,
    Light,
}

impl Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::Dark => write!(f, "dark"),
            Theme::Light => write!(f, "light"),
        }
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate {
    theme: Theme,
    user: User,
}

async fn home(user: Option<CurrentUser>) -> Result<HomeTemplate, Redirect> {
    match user {
        Some(CurrentUser(user)) => Ok(HomeTemplate {
            theme: Theme::Dark,
            user,
        }),
        _ => Err(Redirect::temporary("/login")),
    }
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    theme: Theme,
}

async fn login(user: Option<CurrentUser>) -> Result<LoginTemplate, Redirect> {
    match user {
        Some(_) => Err(Redirect::temporary("/")),
        None => Ok(LoginTemplate { theme: Theme::Dark }),
    }
}
