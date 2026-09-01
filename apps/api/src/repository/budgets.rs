use crate::{
    database::models::budgets::BudgetRow,
    model::budgets::BudgetUpsertRequest,
    utils::error::{AppError, AppResult},
};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct BudgetRepository {
    pool: SqlitePool,
}

impl BudgetRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    pub async fn find_all(&self) -> AppResult<Vec<BudgetRow>> {
        sqlx::query_as(include_str!("../../queries/budgets/find_all.sql"))
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn find_by_id(&self, id: &str) -> AppResult<Option<BudgetRow>> {
        sqlx::query_as(include_str!("../../queries/budgets/find_by_id.sql"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn insert(&self, v: &BudgetUpsertRequest) -> AppResult<BudgetRow> {
        sqlx::query_as(include_str!("../../queries/budgets/insert.sql"))
            .bind(&v.category_id)
            .bind(&v.amount)
            .fetch_one(&self.pool)
            .await
            .map_err(map_unique)
    }
    pub async fn update(&self, id: &str, v: &BudgetUpsertRequest) -> AppResult<Option<BudgetRow>> {
        sqlx::query_as(include_str!("../../queries/budgets/update.sql"))
            .bind(&v.category_id)
            .bind(&v.amount)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_unique)
    }
    pub async fn delete(&self, id: &str) -> AppResult<bool> {
        Ok(
            sqlx::query(include_str!("../../queries/budgets/delete_by_id.sql"))
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected()
                > 0,
        )
    }
    pub async fn category_exists(&self, id: &str) -> AppResult<bool> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM expense_categories WHERE id = ?1)")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(Into::into)
    }
}

fn map_unique(error: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(database) = &error
        && database.is_unique_violation()
    {
        return AppError::bad_request("a budget already exists for this category");
    }
    error.into()
}
