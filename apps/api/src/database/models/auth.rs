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

#[derive(Debug, Clone, FromRow)]
pub struct PasskeyRow {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub credential: String,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
}
