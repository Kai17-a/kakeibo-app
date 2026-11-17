use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: String,
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct SessionRow {
    pub user_id: String,
    pub username: String,
    pub last_seen_at: i64,
    pub expires_at: i64,
}
