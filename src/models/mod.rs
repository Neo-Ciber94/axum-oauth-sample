#[derive(serde::Serialize)]
pub struct User {
    pub id: String,
    pub account_id: String,
    pub username: String,
}

#[derive(serde::Serialize)]
pub struct UserSession {
    pub id: String,
    pub user_id: String,
    pub created_at: u64,
    pub expires_at: u64,
}
