use crate::{
    database::models::transfers::TransferRow, model::transfers::TransferUpsertRequest,
    utils::error::AppResult,
};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct TransferRepository {
    pool: SqlitePool,
}

impl TransferRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    pub async fn find_all(&self) -> AppResult<Vec<TransferRow>> {
        Ok(
            sqlx::query_as(include_str!("../../queries/transfers/find_all.sql"))
                .fetch_all(&self.pool)
                .await?,
        )
    }
    pub async fn find_by_id(&self, id: &str) -> AppResult<Option<TransferRow>> {
        Ok(
            sqlx::query_as(include_str!("../../queries/transfers/find_by_id.sql"))
                .bind(id)
                .fetch_optional(&self.pool)
                .await?,
        )
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
    pub async fn insert(&self, value: &TransferUpsertRequest) -> AppResult<TransferRow> {
        Ok(bind(
            sqlx::query_as(include_str!("../../queries/transfers/insert.sql")),
            value,
        )
        .fetch_one(&self.pool)
        .await?)
    }
    pub async fn update(
        &self,
        id: &str,
        value: &TransferUpsertRequest,
    ) -> AppResult<Option<TransferRow>> {
        Ok(bind(
            sqlx::query_as(include_str!("../../queries/transfers/update.sql")),
            value,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?)
    }
    pub async fn delete(&self, id: &str) -> AppResult<bool> {
        Ok(
            sqlx::query(include_str!("../../queries/transfers/delete_by_id.sql"))
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected()
                > 0,
        )
    }
}

fn bind<'q>(
    query: sqlx::query::QueryAs<'q, sqlx::Sqlite, TransferRow, sqlx::sqlite::SqliteArguments>,
    value: &'q TransferUpsertRequest,
) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, TransferRow, sqlx::sqlite::SqliteArguments> {
    query
        .bind(&value.transaction_date)
        .bind(&value.amount)
        .bind(&value.from_payment_method_id)
        .bind(&value.to_payment_method_id)
        .bind(&value.description)
}
