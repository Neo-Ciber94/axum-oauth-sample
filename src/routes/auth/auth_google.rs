use std::time::Duration;

use axum::{
    extract::Query,
    http::StatusCode,
    response::{ErrorResponse, IntoResponse, Redirect},
    routing::get,
    Extension, Router,
};

use oauth2::{
    basic::BasicClient, AuthorizationCode, CsrfToken, PkceCodeChallenge, Scope, TokenResponse,
};
use oauth2::{reqwest::async_http_client, PkceCodeVerifier};
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};

use crate::constants::{COOKIE_AUTH_CODE_VERIFIER, COOKIE_AUTH_CSRF_STATE, COOKIE_AUTH_SESSION};
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

fn get_auth_client() -> BasicClient {
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

    let base_url = std::env::var("BASE_URL").expect("Failed to get app base url");
    let redirect_url = RedirectUrl::new(format!("{base_url}/api/auth/google/callback"))
        .expect("Invalid redirect url");

    BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(redirect_url)
}

async fn login() -> Result<impl IntoResponse, ErrorResponse> {
    let client = get_auth_client();
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.profile".to_string(),
        ))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    // Set csrf and code verifier cookies
    let csrf_cookie: Cookie =
        Cookie::build((COOKIE_AUTH_CSRF_STATE, csrf_state.secret().to_owned()))
            .http_only(true)
            .path("/")
            .same_site(SameSite::Lax)
            .max_age(cookie::time::Duration::hours(1))
            .into();

    let code_verifier: Cookie = Cookie::build((
        COOKIE_AUTH_CODE_VERIFIER,
        pkce_code_verifier.secret().to_owned(),
    ))
    .http_only(true)
    .path("/")
    .same_site(SameSite::Lax)
    .max_age(cookie::time::Duration::hours(1))
    .into();

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
    let stored_state = cookies.get(COOKIE_AUTH_CSRF_STATE);
    let stored_code_verifier = cookies.get(COOKIE_AUTH_CODE_VERIFIER);

    let (Some(csrf_state), Some(code_verifier)) = (stored_state, stored_code_verifier) else {
        return Err(ErrorResponse::from(StatusCode::BAD_REQUEST));
    };

    if csrf_state.value() != state {
        return Err(ErrorResponse::from(StatusCode::BAD_REQUEST));
    }

    let client = get_auth_client();
    let code = AuthorizationCode::new(code);
    let pkce_code_verifier = PkceCodeVerifier::new(code_verifier.value().to_owned());

    let token_response = client
        .exchange_code(code)
        .set_pkce_verifier(pkce_code_verifier)
        .request_async(async_http_client)
        .await
        .map_err(|_| ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR))?;

    let google_user = fetch_google_user(token_response.access_token().secret())
        .await
        .map_err(|_| ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR))?;

    println!("Get/create user");
    // Add user session
    let account_id = google_user.sub.clone();
    let existing_user = crate::db::get_user_by_account_id(&pool, account_id.clone())
        .await
        .map_err(|_| ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR))?;

    let user = match existing_user {
        Some(x) => x,
        None => crate::db::create_user(
            &pool,
            account_id,
            google_user.name,
            Some(google_user.picture),
        )
        .await
        .map_err(|_| ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR))?,
    };

    let session_duration = Duration::from_millis(1000 * 60 * 60 * 24); // 1 day;
    let user_session = crate::db::create_user_session(&pool, user.id, session_duration)
        .await
        .map_err(|_| ErrorResponse::from(StatusCode::INTERNAL_SERVER_ERROR))?;

    // Remove code_verifier and csrf_state cookies
    let mut remove_csrf_cookie = Cookie::new(COOKIE_AUTH_CSRF_STATE, "");
    remove_csrf_cookie.set_path("/");
    remove_csrf_cookie.make_removal();

    let mut remove_code_verifier = Cookie::new(COOKIE_AUTH_CODE_VERIFIER, "");
    remove_code_verifier.set_path("/");
    remove_code_verifier.make_removal();

    let session_cookie: Cookie = Cookie::build((COOKIE_AUTH_SESSION, user_session.id.to_string()))
        .same_site(SameSite::Lax)
        .http_only(true)
        .path("/")
        .max_age(cookie::time::Duration::milliseconds(
            session_duration.as_millis() as i64,
        ))
        .into();

    let cookies = CookieJar::new()
        .add(remove_csrf_cookie)
        .add(remove_code_verifier)
        .add(session_cookie);

    let response = (cookies, Redirect::temporary("/")).into_response();
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