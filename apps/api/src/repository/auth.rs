use crate::{
    database::models::auth::{SessionRow, UserRow},
    utils::error::{AppError, AppResult},
};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AuthRepository {
    pool: SqlitePool,
}

impl AuthRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_user(&self, username: &str) -> AppResult<Option<UserRow>> {
        sqlx::query_as(include_str!("../../queries/auth/find_user_by_username.sql"))
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn find_all_users(&self) -> AppResult<Vec<UserRow>> {
        sqlx::query_as(include_str!("../../queries/auth/find_all_users.sql"))
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn count_users(&self) -> AppResult<i64> {
        sqlx::query_scalar(include_str!("../../queries/auth/count_users.sql"))
            .fetch_one(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn insert_user(&self, username: &str, password_hash: &str) -> AppResult<()> {
        sqlx::query(include_str!("../../queries/auth/insert_user.sql"))
            .bind(username)
            .bind(password_hash)
            .execute(&self.pool)
            .await
            .map_err(|error| match &error {
                sqlx::Error::Database(database) if database.is_unique_violation() => {
                    AppError::conflict("username is already registered")
                }
                _ => error.into(),
            })?;
        Ok(())
    }

    /// Replaces the password and signs the user out everywhere. Returns false when the user
    /// does not exist.
    pub async fn update_password(&self, username: &str, password_hash: &str) -> AppResult<bool> {
        let mut transaction = self.pool.begin().await?;
        let updated = sqlx::query(include_str!("../../queries/auth/update_password.sql"))
            .bind(username)
            .bind(password_hash)
            .execute(&mut *transaction)
            .await?
            .rows_affected()
            > 0;
        sqlx::query(include_str!("../../queries/auth/delete_user_sessions.sql"))
            .bind(username)
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;
        Ok(updated)
    }

    pub async fn delete_user(&self, username: &str) -> AppResult<bool> {
        Ok(
            sqlx::query(include_str!("../../queries/auth/delete_user.sql"))
                .bind(username)
                .execute(&self.pool)
                .await?
                .rows_affected()
                > 0,
        )
    }

    pub async fn insert_session(
        &self,
        token_hash: &str,
        user_id: &str,
        now: i64,
        expires_at: i64,
    ) -> AppResult<()> {
        sqlx::query(include_str!("../../queries/auth/insert_session.sql"))
            .bind(token_hash)
            .bind(user_id)
            .bind(now)
            .bind(expires_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn find_session(&self, token_hash: &str) -> AppResult<Option<SessionRow>> {
        sqlx::query_as(include_str!("../../queries/auth/find_session.sql"))
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn touch_session(
        &self,
        token_hash: &str,
        now: i64,
        expires_at: i64,
    ) -> AppResult<()> {
        sqlx::query(include_str!("../../queries/auth/touch_session.sql"))
            .bind(token_hash)
            .bind(now)
            .bind(expires_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_session(&self, token_hash: &str) -> AppResult<()> {
        sqlx::query(include_str!("../../queries/auth/delete_session.sql"))
            .bind(token_hash)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_expired_sessions(&self, now: i64) -> AppResult<u64> {
        Ok(sqlx::query(include_str!(
            "../../queries/auth/delete_expired_sessions.sql"
        ))
        .bind(now)
        .execute(&self.pool)
        .await?
        .rows_affected())
    }
}
