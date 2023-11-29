use askama::Template;
use axum::{routing::get, Router};

pub fn pages_router() -> Router {
    Router::new().route("/", get(hello))
}

#[derive(Template)]
#[template(path = "index.html")]
struct HelloTemplate<'a> {
    title: &'a str,
    name: &'a str,
}

async fn hello() -> HelloTemplate<'static> {
    return HelloTemplate {
        title: "Home Page",
        name: "Lilly",
    };
}
