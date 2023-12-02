use std::{str::FromStr, time::Duration};

use crate::models::{User, UserSession};
use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn get_user_by_account_id(
    pool: &SqlitePool,
    account_id: String,
) -> Result<Option<User>, anyhow::Error> {
    let user = sqlx::query_as!(
        User,
        r#"
            SELECT id as "id: uuid::Uuid", account_id, username, image_url
            FROM user 
            WHERE account_id = ?1
        "#,
        account_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_session_id(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Option<User>, anyhow::Error> {
    let session_id = Uuid::from_str(session_id)?;
    let user = sqlx::query_as!(
        User,
        r#"
            SELECT user.id as "id: uuid::Uuid", account_id, username, image_url
            FROM user
            LEFT JOIN user_session AS session ON session.user_id = user.id
            WHERE session.id = ?1
        "#,
        session_id
    )
    .fetch_optional(pool)
    .await?;

    if let Some(user) = &user {
        let deleted = delete_expired_user_sessions(pool, user.id).await?;
        if deleted > 0 {
            tracing::info!(
                "{deleted:?} expired sessions where deleted for user '{}'",
                user.id
            )
        }
    }

    Ok(user)
}

pub async fn create_user(
    pool: &SqlitePool,
    account_id: String,
    username: String,
    image_url: Option<String>,
) -> Result<User, anyhow::Error> {
    let id = Uuid::new_v4();
    let new_user = sqlx::query_as!(
        User,
        r#"
            INSERT INTO user (id, account_id, username, image_url) 
            VALUES (?1, ?2, ?3, ?4) 
            RETURNING id as "id: uuid::Uuid", account_id, username, image_url
        "#,
        id,
        account_id,
        username,
        image_url
    )
    .fetch_one(pool)
    .await?;

    Ok(new_user)
}

pub async fn create_user_session(
    pool: &SqlitePool,
    user_id: Uuid,
    session_duration: Duration,
) -> Result<UserSession, anyhow::Error> {
    let session_id = Uuid::new_v4();
    let created_at = chrono::offset::Utc::now();
    let expires_at = created_at + session_duration;

    let inserted = sqlx::query!(
        r#"
            INSERT INTO user_session (id, user_id, created_at, expires_at)
            VALUES (?1, ?2, ?3, ?4)
            RETURNING
            id as "id: uuid::Uuid"
        "#,
        session_id,
        user_id,
        created_at,
        expires_at
    )
    .fetch_one(pool)
    .await?;

    let user_session = sqlx::query_as!(
        UserSession,
        r#"
            SELECT 
                id as "id: uuid::Uuid",
                user_id as "user_id: uuid::Uuid",
                created_at as "created_at: _",
                expires_at as "expires_at: _" 
            FROM user_session
            WHERE id = ?1
        "#,
        inserted.id
    )
    .fetch_one(pool)
    .await?;

    Ok(user_session)
}

pub async fn delete_user_session(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<bool, anyhow::Error> {
    let session_id = Uuid::from_str(session_id)?;
    let mut conn = pool.acquire().await?;

    let result = sqlx::query!("DELETE FROM user_session WHERE id = ?1", session_id)
        .execute(&mut *conn)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_expired_user_sessions(
    pool: &SqlitePool,
    user_id: Uuid,
) -> Result<usize, anyhow::Error> {
    let now = chrono::offset::Utc::now();
    let result = sqlx::query!(
        r#"
            DELETE FROM user_session 
            WHERE user_id = ?1 AND ?2 > expires_at
        "#,
        user_id,
        now
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() as usize)
}
