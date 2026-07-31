use crate::{
    database::models::recurring_expenses::RecurringExpenseRow,
    model::recurring_expenses::RecurringExpenseUpsertRequest, utils::error::AppResult,
};
use sqlx::SqlitePool;
#[derive(Clone)]
pub struct RecurringExpenseRepository {
    pool: SqlitePool,
}
impl RecurringExpenseRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    pub async fn find_all(&self) -> AppResult<Vec<RecurringExpenseRow>> {
        sqlx::query_as(include_str!(
            "../../queries/recurring_expenses/find_all.sql"
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }
    pub async fn find_by_id(&self, id: &str) -> AppResult<Option<RecurringExpenseRow>> {
        sqlx::query_as(include_str!(
            "../../queries/recurring_expenses/find_by_id.sql"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }
    pub async fn insert(
        &self,
        v: &RecurringExpenseUpsertRequest,
    ) -> AppResult<RecurringExpenseRow> {
        bind(
            sqlx::query_as(include_str!("../../queries/recurring_expenses/insert.sql")),
            v,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }
    pub async fn update(
        &self,
        id: &str,
        v: &RecurringExpenseUpsertRequest,
    ) -> AppResult<Option<RecurringExpenseRow>> {
        bind(
            sqlx::query_as(include_str!("../../queries/recurring_expenses/update.sql")),
            v,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }
    pub async fn delete(&self, id: &str) -> AppResult<bool> {
        Ok(sqlx::query(include_str!(
            "../../queries/recurring_expenses/delete_by_id.sql"
        ))
        .bind(id)
        .execute(&self.pool)
        .await?
        .rows_affected()
            > 0)
    }
}
fn bind<'q>(
    q: sqlx::query::QueryAs<'q, sqlx::Sqlite, RecurringExpenseRow, sqlx::sqlite::SqliteArguments>,
    v: &'q RecurringExpenseUpsertRequest,
) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, RecurringExpenseRow, sqlx::sqlite::SqliteArguments> {
    q.bind(&v.name)
        .bind(&v.amount)
        .bind(v.payment_day)
        .bind(&v.start_date)
        .bind(&v.end_date)
        .bind(&v.category_id)
        .bind(&v.payment_method_id)
        .bind(v.is_active)
        .bind(v.is_variable)
        .bind(&v.description)
}
