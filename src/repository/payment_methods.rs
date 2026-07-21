use crate::{
    database::models::payment_methods::PaymentMethodRow,
    model::payment_methods::{
        PaymentMethodQuery, PaymentMethodSortBy, PaymentMethodSortOrder, PaymentMethodUpsertRequest,
    },
    utils::error::AppResult,
};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
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
            "SELECT id,created_at,updated_at,name,description FROM payment_methods WHERE 1=1",
        );
        if let Some(id) = &q.id {
            b.push(" AND id = ").push_bind(id);
        }
        let sort = q.sort_by.unwrap_or(PaymentMethodSortBy::Name);
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
}
