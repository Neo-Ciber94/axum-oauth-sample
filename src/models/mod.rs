use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, serde::Serialize)]
pub struct User {
    pub id: Uuid,
    pub account_id: String,
    pub provider: AuthProvider,
    pub username: String,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
struct UnknownProvider {
    _priv: (),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum AuthProvider {
    Google,
    Github,
    Discord,

    // This variant should not be constructed
    #[allow(private_interfaces)]
    Unknown(UnknownProvider),
}

impl From<String> for AuthProvider {
    fn from(value: String) -> Self {
        match value.as_str() {
            "google" => AuthProvider::Google,
            "github" => AuthProvider::Github,
            "discord" => AuthProvider::Discord,
            _ => AuthProvider::Unknown(UnknownProvider { _priv: () }),
        }
    }
}

impl std::fmt::Display for AuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthProvider::Google => write!(f, "google"),
            AuthProvider::Github => write!(f, "github"),
            AuthProvider::Discord => write!(f, "discord"),
            _ => write!(f, "unknown provider"),
        }
    }
}
