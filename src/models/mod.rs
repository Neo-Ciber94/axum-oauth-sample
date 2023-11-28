use uuid::Uuid;

#[serde::Serialize]
pub struct User {
    pub id: Uuid,
    pub username: String,
}

pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: u64,
    pub expires_at: u64,
}
