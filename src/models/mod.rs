use uuid::Uuid;

#[derive(serde::Serialize)]
pub struct User {
    pub id: Uuid,
    pub account_id: String,
    pub username: String,
}

#[derive(serde::Serialize)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: u64,
    pub expires_at: u64,
}
