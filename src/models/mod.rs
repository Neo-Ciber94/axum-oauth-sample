use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, serde::Serialize)]
pub struct User {
    pub id: Uuid,
    pub account_id: String,
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
