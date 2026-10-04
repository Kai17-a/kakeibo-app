use crate::{
    database::models::recurring_transfers::RecurringTransferRow,
    model::recurring_transfers::RecurringTransferUpsertRequest, utils::error::AppResult,
};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct RecurringTransferRepository {
    pool: SqlitePool,
}

impl RecurringTransferRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    pub async fn find_all(&self) -> AppResult<Vec<RecurringTransferRow>> {
        Ok(sqlx::query_as(include_str!(
            "../../queries/recurring_transfers/find_all.sql"
        ))
        .fetch_all(&self.pool)
        .await?)
    }
    pub async fn find_by_id(&self, id: &str) -> AppResult<Option<RecurringTransferRow>> {
        Ok(sqlx::query_as(include_str!(
            "../../queries/recurring_transfers/find_by_id.sql"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await?)
    }
    pub async fn payment_method_exists(&self, id: &str) -> AppResult<bool> {
        Ok(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM payment_methods WHERE id = ?")
                .bind(id)
                .fetch_one(&self.pool)
                .await?
                > 0,
        )
    }
    pub async fn is_referenced_by_transfer(&self, id: &str) -> AppResult<bool> {
        Ok(sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM transfers WHERE recurring_transfer_id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?
            > 0)
    }
    pub async fn insert(
        &self,
        value: &RecurringTransferUpsertRequest,
    ) -> AppResult<RecurringTransferRow> {
        Ok(bind(
            sqlx::query_as(include_str!("../../queries/recurring_transfers/insert.sql")),
            value,
        )
        .fetch_one(&self.pool)
        .await?)
    }
    pub async fn update(
        &self,
        id: &str,
        value: &RecurringTransferUpsertRequest,
    ) -> AppResult<Option<RecurringTransferRow>> {
        Ok(bind(
            sqlx::query_as(include_str!("../../queries/recurring_transfers/update.sql")),
            value,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?)
    }
    pub async fn delete(&self, id: &str) -> AppResult<bool> {
        Ok(sqlx::query(include_str!(
            "../../queries/recurring_transfers/delete_by_id.sql"
        ))
        .bind(id)
        .execute(&self.pool)
        .await?
        .rows_affected()
            > 0)
    }
}

fn bind<'q>(
    query: sqlx::query::QueryAs<
        'q,
        sqlx::Sqlite,
        RecurringTransferRow,
        sqlx::sqlite::SqliteArguments,
    >,
    value: &'q RecurringTransferUpsertRequest,
) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, RecurringTransferRow, sqlx::sqlite::SqliteArguments> {
    query
        .bind(&value.name)
        .bind(&value.amount)
        .bind(value.payment_day)
        .bind(&value.start_date)
        .bind(&value.end_date)
        .bind(&value.from_payment_method_id)
        .bind(&value.to_payment_method_id)
        .bind(value.is_active)
        .bind(&value.description)
}
