use crate::{
    database::models::expenses::ExpenseRow, model::expenses::ExpenseUpsertRequest,
    utils::error::AppResult,
};
use sqlx::SqlitePool;
#[derive(Clone)]
pub struct ExpenseRepository {
    pool: SqlitePool,
}
impl ExpenseRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    pub async fn find_all(&self) -> AppResult<Vec<ExpenseRow>> {
        sqlx::query_as(include_str!("../../queries/expenses/find_all.sql"))
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn find_by_id(&self, id: &str) -> AppResult<Option<ExpenseRow>> {
        sqlx::query_as(include_str!("../../queries/expenses/find_by_id.sql"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn insert(&self, v: &ExpenseUpsertRequest) -> AppResult<ExpenseRow> {
        bind(
            sqlx::query_as(include_str!("../../queries/expenses/insert.sql")),
            v,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }
    pub async fn update(
        &self,
        id: &str,
        v: &ExpenseUpsertRequest,
    ) -> AppResult<Option<ExpenseRow>> {
        bind(
            sqlx::query_as(include_str!("../../queries/expenses/update.sql")),
            v,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }
    pub async fn delete(&self, id: &str) -> AppResult<bool> {
        Ok(
            sqlx::query(include_str!("../../queries/expenses/delete_by_id.sql"))
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected()
                > 0,
        )
    }
}
fn bind<'q>(
    q: sqlx::query::QueryAs<'q, sqlx::Sqlite, ExpenseRow, sqlx::sqlite::SqliteArguments>,
    v: &'q ExpenseUpsertRequest,
) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, ExpenseRow, sqlx::sqlite::SqliteArguments> {
    q.bind(&v.transaction_date)
        .bind(&v.amount)
        .bind(&v.category_id)
        .bind(&v.payment_method_id)
        .bind(&v.recurring_expense_id)
        .bind(&v.description)
}
