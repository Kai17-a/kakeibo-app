use crate::{
    database::models::recurring_incomes::RecurringIncomeRow,
    model::recurring_incomes::RecurringIncomeUpsertRequest, utils::error::AppResult,
};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct RecurringIncomeRepository {
    pool: SqlitePool,
}

impl RecurringIncomeRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    pub async fn find_all(&self) -> AppResult<Vec<RecurringIncomeRow>> {
        sqlx::query_as(include_str!("../../queries/recurring_incomes/find_all.sql"))
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn find_by_id(&self, id: &str) -> AppResult<Option<RecurringIncomeRow>> {
        sqlx::query_as(include_str!(
            "../../queries/recurring_incomes/find_by_id.sql"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }
    pub async fn insert(&self, v: &RecurringIncomeUpsertRequest) -> AppResult<RecurringIncomeRow> {
        bind(
            sqlx::query_as(include_str!("../../queries/recurring_incomes/insert.sql")),
            v,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }
    pub async fn update(
        &self,
        id: &str,
        v: &RecurringIncomeUpsertRequest,
    ) -> AppResult<Option<RecurringIncomeRow>> {
        bind(
            sqlx::query_as(include_str!("../../queries/recurring_incomes/update.sql")),
            v,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }
    pub async fn delete(&self, id: &str) -> AppResult<bool> {
        Ok(sqlx::query(include_str!(
            "../../queries/recurring_incomes/delete_by_id.sql"
        ))
        .bind(id)
        .execute(&self.pool)
        .await?
        .rows_affected()
            > 0)
    }
}

fn bind<'q>(
    q: sqlx::query::QueryAs<'q, sqlx::Sqlite, RecurringIncomeRow, sqlx::sqlite::SqliteArguments>,
    v: &'q RecurringIncomeUpsertRequest,
) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, RecurringIncomeRow, sqlx::sqlite::SqliteArguments> {
    q.bind(&v.name)
        .bind(&v.amount)
        .bind(v.payment_day)
        .bind(&v.start_date)
        .bind(&v.end_date)
        .bind(&v.category_id)
        .bind(v.is_active)
        .bind(v.is_variable)
        .bind(&v.description)
}
