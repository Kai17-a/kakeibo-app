use crate::{
    database::models::export::{ExpenseExportRow, IncomeExportRow},
    utils::error::AppResult,
};
use sqlx::SqlitePool;
#[derive(Clone)]
pub struct ExportRepository {
    pool: SqlitePool,
}
impl ExportRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    pub async fn find_expenses(&self) -> AppResult<Vec<ExpenseExportRow>> {
        sqlx::query_as(include_str!("../../queries/export/expenses.sql"))
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn find_incomes(&self) -> AppResult<Vec<IncomeExportRow>> {
        sqlx::query_as(include_str!("../../queries/export/incomes.sql"))
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }
}
