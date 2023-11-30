use askama::Template;
use axum::{response::Redirect, routing::get, Router};

use crate::{models::User, server::CurrentUser};

pub fn pages_router() -> Router {
    Router::new()
        .route("/", get(home))
        .route("/login", get(login))
}

#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate {
    user: User,
}

async fn home(user: Option<CurrentUser>) -> Result<HomeTemplate, Redirect> {
    match user {
        Some(CurrentUser(user)) => Ok(HomeTemplate { user }),
        _ => Err(Redirect::temporary("/login")),
    }
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate;

async fn login(user: Option<CurrentUser>) -> Result<LoginTemplate, Redirect> {
    match user {
        Some(_) => Err(Redirect::temporary("/")),
        None => Ok(LoginTemplate),
    }
}
