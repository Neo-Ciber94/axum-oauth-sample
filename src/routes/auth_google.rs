use std::{ops::Add, time::Duration};

use axum::{
    extract::Query,
    http::{header, HeaderMap, StatusCode},
    response::{ErrorResponse, IntoResponse, Redirect},
    routing::{get, post},
    Extension, Router,
};

use oauth2::{
    basic::BasicClient, AuthorizationCode, CsrfToken, PkceCodeChallenge, Scope, TokenResponse,
};
use oauth2::{reqwest::async_http_client, PkceCodeVerifier};
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use uuid::Uuid;

use crate::models::{User, UserSession};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use sqlx::SqlitePool;

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct GoogleUser {
    sub: String,
    name: String,
    email: Option<String>,
    email_verified: Option<String>,
    picture: String,
}

pub fn google_auth_router() -> Router {
    Router::new()
        .route("/google/login", get(login))
        .route("/google/callback", get(callback))
}

fn get_client() -> Result<BasicClient, Box<dyn std::error::Error>> {
    let client_id = ClientId::new(
        std::env::var("GOOGLE_CLIENT_ID")
            .expect("Missing the GOOGLE_CLIENT_ID environment variable."),
    );

    let client_secret = ClientSecret::new(
        std::env::var("GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
    );

    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        .expect("Invalid token endpoint URL");

    Ok(
        BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
            // Set the URL the user will be redirected to after the authorization process.
            .set_redirect_uri(RedirectUrl::new(
                "http://localhost:3000/auth/google/callback".to_string(),
            )?),
    )
}

async fn login() -> Result<impl IntoResponse, ErrorResponse> {
    let client =
        get_client().map_err(|_| ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR))?;
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        // This example is requesting access to the "calendar" features and the user's profile.
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.profile".to_string(),
        ))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    // Set auth cookies
    let mut csrf_cookie = Cookie::new("auth_csrf_state", csrf_state.secret().to_owned());
    csrf_cookie.set_http_only(true);
    csrf_cookie.set_path("/");
    //csrf_cookie.set_same_site(Some(SameSite::Strict));
    csrf_cookie.set_max_age(cookie::time::Duration::hours(1));

    let mut code_verifier =
        Cookie::new("auth_code_verifier", pkce_code_verifier.secret().to_owned());
    code_verifier.set_http_only(true);
    code_verifier.set_path("/");
    //code_verifier.set_same_site(Some(SameSite::Strict));
    code_verifier.set_max_age(cookie::time::Duration::hours(1));

    let cookies = CookieJar::new().add(csrf_cookie).add(code_verifier);

    Ok((cookies, Redirect::temporary(authorize_url.as_str())))
}

#[derive(serde::Deserialize)]
struct GoogleCallbackQuery {
    code: String,
    state: String,
}

async fn callback(
    cookies: CookieJar,
    Extension(pool): Extension<SqlitePool>,
    Query(query): Query<GoogleCallbackQuery>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let code = query.code;
    let state = query.state;
    let stored_state = cookies.get("auth_csrf_state");
    let stored_code_verifier = cookies.get("auth_code_verifier");

    let (Some(csrf_state), Some(code_verifier)) = (stored_state, stored_code_verifier) else {
        eprintln!("Failed to get auth cookies");
        return Err(ErrorResponse::from(StatusCode::BAD_REQUEST));
    };

    if csrf_state.value() != state {
        eprintln!("Invalid csrf state");
        return Err(ErrorResponse::from(StatusCode::BAD_REQUEST));
    }

    println!("Get google client");
    let client =
        get_client().map_err(|_| ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR))?;
    let code = AuthorizationCode::new(code);
    let pkce_code_verifier = PkceCodeVerifier::new(code_verifier.value().to_owned());

    println!("Get token response");

    let token_response = client
        .exchange_code(code)
        .set_pkce_verifier(pkce_code_verifier)
        .request_async(async_http_client)
        .await
        .map_err(|_| ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR))?;

    println!("Fetch google user");

    let google_user = fetch_google_user(token_response.access_token().secret())
        .await
        .map_err(|_| ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR))?;

    println!("Search existing user");

    // Add user session
    let account_id = google_user.sub.clone();
    let existing_user =
        sqlx::query_as!(User, "SELECT * FROM user WHERE account_id = ?1", account_id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR))?;

    let mut conn = pool
        .acquire()
        .await
        .map_err(|_| ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR))?;

    println!("Try insert user");

    let user_id = match existing_user {
        Some(user) => user.id,
        None => {
            let id = Uuid::new_v4().to_string();
            sqlx::query!(
                "INSERT INTO user (id, account_id, username) VALUES (?1, ?2, ?3)",
                id,
                google_user.sub,
                google_user.name
            )
            .execute(&mut *conn)
            .await
            .map_err(|_| ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR))?;

            id
        }
    };

    let session_id = Uuid::new_v4().to_string();
    let created_at = now();
    let expires_at = created_at.add(Duration::from_secs(60 * 60 * 24)); // 1 day

    // FIXME: Store as u128 instead
    let created_at_ms = created_at.as_millis() as i64;
    let expires_at_ms = expires_at.as_millis() as i64;

    println!("Try session user");
    sqlx::query!(
        "INSERT INTO user_session (id, user_id, created_at, expires_at) VALUES (?1, ?2, ?3, ?4)",
        session_id,
        user_id,
        created_at_ms,
        expires_at_ms
    )
    .execute(&mut *conn)
    .await
    .map_err(|_| ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR))?;

    drop(conn);

    // Remove code_verifier and csrf_state cookies
    let mut remove_csrf_cookie = Cookie::new("auth_csrf_state", "");
    let mut remove_code_verifier = Cookie::new("auth_code_verifier", "");

    remove_csrf_cookie.make_removal();
    remove_code_verifier.make_removal();

    let mut session_cookie = Cookie::new("auth_session", session_id);
    //session_cookie.set_same_site(Some(SameSite::Strict));
    session_cookie.set_http_only(true);
    session_cookie.set_path("/");
    session_cookie.set_max_age(cookie::time::Duration::hours(24)); // TODO: Is this correct?

    let cookies = CookieJar::new()
        .add(remove_csrf_cookie)
        .add(remove_code_verifier)
        .add(session_cookie);

    let response = (cookies, Redirect::temporary("/auth/me")).into_response();
    Ok(response)
}

async fn fetch_google_user(access_token: &str) -> Result<GoogleUser, reqwest::Error> {
    reqwest::Client::new()
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(access_token)
        .send()
        .await?
        .json::<GoogleUser>()
        .await
}

fn now() -> Duration {
    use std::time::SystemTime;
    let now = SystemTime::now();
    now.elapsed().unwrap()
}
