use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::{
    database::models::incomes::IncomeRow,
    model::incomes::{IncomeQuery, IncomeSortBy, IncomeUpsertRequest, SortOrder},
    utils::error::AppResult,
};

#[derive(Clone)]
pub struct IncomeRepository {
    pool: SqlitePool,
}

impl IncomeRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_all(&self, query: &IncomeQuery) -> AppResult<Vec<IncomeRow>> {
        let mut builder = filtered_query(
            "SELECT id, created_at, updated_at, category_id, transaction_date, amount, \
             payment_method_id, recurring_income_id, description FROM incomes WHERE 1 = 1",
            query,
        );

        let sort_by = query.sort_by.unwrap_or(IncomeSortBy::TransactionDate);
        let sort_order = query.sort_order.unwrap_or(SortOrder::Desc);
        let page = query.page();
        let per_page = query.per_page();
        let offset = u64::from(page - 1) * u64::from(per_page);

        builder
            .push(" ORDER BY ")
            .push(sort_by.sql())
            .push(" ")
            .push(sort_order.sql())
            .push(", id ASC LIMIT ")
            .push_bind(i64::from(per_page))
            .push(" OFFSET ")
            .push_bind(offset as i64);

        builder
            .build_query_as()
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn find_by_id(&self, id: &str) -> AppResult<Option<IncomeRow>> {
        sqlx::query_as(include_str!("../../queries/incomes/find_by_id.sql"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn insert(&self, input: &IncomeUpsertRequest) -> AppResult<IncomeRow> {
        sqlx::query_as(include_str!("../../queries/incomes/insert.sql"))
            .bind(&input.category_id)
            .bind(&input.transaction_date)
            .bind(&input.amount)
            .bind(&input.payment_method_id)
            .bind(&input.recurring_income_id)
            .bind(&input.description)
            .fetch_one(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn update(
        &self,
        id: &str,
        input: &IncomeUpsertRequest,
    ) -> AppResult<Option<IncomeRow>> {
        sqlx::query_as(include_str!("../../queries/incomes/update.sql"))
            .bind(&input.category_id)
            .bind(&input.transaction_date)
            .bind(&input.amount)
            .bind(&input.payment_method_id)
            .bind(&input.recurring_income_id)
            .bind(&input.description)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn delete(&self, id: &str) -> AppResult<bool> {
        let result = sqlx::query(include_str!("../../queries/incomes/delete_by_id.sql"))
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn current_month(&self) -> AppResult<String> {
        sqlx::query_scalar(include_str!("../../queries/incomes/current_month.sql"))
            .fetch_one(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn insert_recurring_for_month(&self, month: &str) -> AppResult<u64> {
        Ok(sqlx::query(include_str!(
            "../../queries/incomes/insert_recurring_for_month.sql"
        ))
        .bind(month)
        .execute(&self.pool)
        .await?
        .rows_affected())
    }
}

fn filtered_query(select: &'static str, query: &IncomeQuery) -> QueryBuilder<Sqlite> {
    let mut builder = QueryBuilder::new(select);
    if let Some(id) = &query.id {
        builder.push(" AND id = ").push_bind(id);
    }
    if let Some(category_id) = &query.category_id {
        builder.push(" AND category_id = ").push_bind(category_id);
    }
    if let Some(payment_method_id) = &query.payment_method_id {
        builder
            .push(" AND payment_method_id = ")
            .push_bind(payment_method_id);
    }
    if let Some(date_from) = &query.date_from {
        builder
            .push(" AND transaction_date >= ")
            .push_bind(date_from);
    }
    if let Some(date_to) = &query.date_to {
        builder.push(" AND transaction_date <= ").push_bind(date_to);
    }
    builder
}
