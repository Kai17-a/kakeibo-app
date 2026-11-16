use crate::{
    database::models::payment_methods::PaymentMethodRow,
    model::payment_methods::{
        PaymentMethodQuery, PaymentMethodSortBy, PaymentMethodSortOrder, PaymentMethodUpsertRequest,
    },
    utils::error::{AppError, AppResult},
};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use std::collections::HashSet;
#[derive(Clone)]
pub struct PaymentMethodRepository {
    pool: SqlitePool,
}
impl PaymentMethodRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    pub async fn find_all(&self, q: &PaymentMethodQuery) -> AppResult<Vec<PaymentMethodRow>> {
        let mut b = QueryBuilder::<Sqlite>::new(
            "SELECT id,created_at,updated_at,name,description,initial_balance,is_investment,display_order, \
             (SELECT COALESCE(SUM(CAST(amount AS INTEGER)), 0) FROM incomes \
              WHERE payment_method_id = payment_methods.id) AS income_total, \
             (SELECT COALESCE(SUM(CAST(amount AS INTEGER)), 0) FROM expenses \
              WHERE payment_method_id = payment_methods.id) AS expense_total, \
             (SELECT COALESCE(SUM(CAST(amount AS INTEGER)), 0) FROM transfers \
              WHERE to_payment_method_id = payment_methods.id) AS transfer_in_total, \
             (SELECT COALESCE(SUM(CAST(amount AS INTEGER)), 0) FROM transfers \
              WHERE from_payment_method_id = payment_methods.id) AS transfer_out_total \
             FROM payment_methods WHERE 1=1",
        );
        if let Some(id) = &q.id {
            b.push(" AND id = ").push_bind(id);
        }
        let sort = q.sort_by.unwrap_or(PaymentMethodSortBy::DisplayOrder);
        let order = q.sort_order.unwrap_or(PaymentMethodSortOrder::Asc);
        b.push(" ORDER BY ")
            .push(sort.sql())
            .push(" ")
            .push(order.sql())
            .push(", id ASC LIMIT ")
            .push_bind(i64::from(q.per_page()))
            .push(" OFFSET ")
            .push_bind((u64::from(q.page() - 1) * u64::from(q.per_page())) as i64);
        b.build_query_as()
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn reorder(&self, ids: &[String]) -> AppResult<()> {
        let mut tx = self.pool.begin().await?;
        let siblings: Vec<String> = sqlx::query_scalar("SELECT id FROM payment_methods")
            .fetch_all(&mut *tx)
            .await?;
        validate_reorder(&siblings, ids)?;
        for (display_order, id) in ids.iter().enumerate() {
            sqlx::query("UPDATE payment_methods SET display_order = ?, updated_at = current_timestamp WHERE id = ?")
                .bind(display_order as i64)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }
    pub async fn find_by_id(&self, id: &str) -> AppResult<Option<PaymentMethodRow>> {
        sqlx::query_as(include_str!("../../queries/payment_methods/find_by_id.sql"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn insert(&self, v: &PaymentMethodUpsertRequest) -> AppResult<PaymentMethodRow> {
        sqlx::query_as(include_str!("../../queries/payment_methods/insert.sql"))
            .bind(&v.name)
            .bind(&v.description)
            .bind(&v.initial_balance)
            .bind(v.is_investment)
            .fetch_one(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn update(
        &self,
        id: &str,
        v: &PaymentMethodUpsertRequest,
    ) -> AppResult<Option<PaymentMethodRow>> {
        sqlx::query_as(include_str!("../../queries/payment_methods/update.sql"))
            .bind(&v.name)
            .bind(&v.description)
            .bind(&v.initial_balance)
            .bind(v.is_investment)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn delete(&self, id: &str) -> AppResult<bool> {
        Ok(sqlx::query(include_str!(
            "../../queries/payment_methods/delete_by_id.sql"
        ))
        .bind(id)
        .execute(&self.pool)
        .await?
        .rows_affected()
            > 0)
    }
    pub async fn is_referenced_by_transactions(&self, id: &str) -> AppResult<bool> {
        let transaction_count: i64 = sqlx::query_scalar(
            "SELECT (SELECT COUNT(*) FROM expenses WHERE payment_method_id = ?1) + \
             (SELECT COUNT(*) FROM incomes WHERE payment_method_id = ?1) + \
             (SELECT COUNT(*) FROM transfers WHERE from_payment_method_id = ?1 OR to_payment_method_id = ?1)",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        if transaction_count > 0 {
            return Ok(true);
        }
        let has_table: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'recurring_transfers')",
        )
        .fetch_one(&self.pool)
        .await?;
        if !has_table {
            return Ok(false);
        }
        Ok(sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM recurring_transfers WHERE from_payment_method_id = ?1 OR to_payment_method_id = ?1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?
            > 0)
    }
}

fn validate_reorder(siblings: &[String], requested: &[String]) -> AppResult<()> {
    let sibling_set: HashSet<_> = siblings.iter().collect();
    let requested_set: HashSet<_> = requested.iter().collect();
    if requested_set.len() != requested.len() {
        return Err(AppError::bad_request(
            "payment_method_ids must not contain duplicates",
        ));
    }
    if sibling_set != requested_set {
        return Err(AppError::bad_request(
            "payment_method_ids must exactly match all payment methods",
        ));
    }
    Ok(())
}
