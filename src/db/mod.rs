use std::time::Duration;

use crate::models::{User, UserSession};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn get_user_by_account_id(
    pool: &SqlitePool,
    account_id: String,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"
            SELECT id as "id: uuid::Uuid", account_id, username
            FROM user 
            WHERE account_id = ?1
        "#,
        account_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_user_by_session_id<S: Into<String>>(
    pool: &SqlitePool,
    session_id: S,
) -> Result<Option<User>, sqlx::Error> {
    let session_id = session_id.into();

    sqlx::query_as!(
        User,
        r#"
                SELECT user.id as "id: _", account_id, username
                FROM user
                LEFT JOIN user_session AS session ON session.user_id = user.id
                WHERE session.id = ?1
            "#,
        session_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn create_user(
    pool: &SqlitePool,
    account_id: String,
    username: String,
) -> Result<User, sqlx::Error> {
    let id = Uuid::new_v4();

    sqlx::query_as!(
        User,
        r#"
            INSERT INTO user (id, account_id, username) 
            VALUES (?1, ?2, ?3) 
            RETURNING id as "id: uuid::Uuid", account_id, username
        "#,
        id,
        account_id,
        username
    )
    .fetch_one(pool)
    .await
}

pub async fn create_user_session(
    pool: &SqlitePool,
    user_id: Uuid,
    session_duration: Duration,
) -> Result<UserSession, sqlx::Error> {
    let session_id = Uuid::new_v4().to_string();
    let created_at = DateTime::<Utc>::default();
    let expires_at = created_at + session_duration;

    sqlx::query_as!(
        UserSession,
        r#"
            INSERT INTO user_session (id, user_id, created_at, expires_at) 
            VALUES (?1, ?2, ?3, ?4)
            RETURNING 
            id as "id: uuid::Uuid", 
            user_id as "user_id: uuid::Uuid", 
            created_at as "created_at: _", 
            expires_at as "expires_at: _"
        "#,
        session_id,
        user_id,
        created_at,
        expires_at
    )
    .fetch_one(pool)
    .await
}

pub async fn delete_user_session<S: Into<String>>(
    pool: &SqlitePool,
    session_id: S,
) -> Result<bool, sqlx::Error> {
    let session_id = session_id.into();
    let mut conn = pool.acquire().await?;

    let result = sqlx::query!("DELETE FROM user_session WHERE id = ?1", session_id)
        .execute(&mut *conn)
        .await?;

    Ok(result.rows_affected() > 0)
}
