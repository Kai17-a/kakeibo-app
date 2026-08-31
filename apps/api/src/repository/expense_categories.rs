use crate::{
    database::models::expense_categories::ExpenseCategoryRow,
    model::expense_categories::{
        ExpenseCategoryQuery, ExpenseCategorySortBy, ExpenseCategorySortOrder,
        ExpenseCategoryUpsertRequest,
    },
    utils::error::AppResult,
};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
#[derive(Clone)]
pub struct ExpenseCategoryRepository {
    pool: SqlitePool,
}
impl ExpenseCategoryRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    pub async fn find_all(&self, q: &ExpenseCategoryQuery) -> AppResult<Vec<ExpenseCategoryRow>> {
        let mut b = QueryBuilder::<Sqlite>::new(
            "SELECT id, created_at, updated_at, name, description, parent_category_id FROM expense_categories WHERE 1=1",
        );
        if let Some(id) = &q.id {
            b.push(" AND id = ").push_bind(id);
        }
        let sort = q.sort_by.unwrap_or(ExpenseCategorySortBy::Name);
        let order = q.sort_order.unwrap_or(ExpenseCategorySortOrder::Asc);
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
    pub async fn find_by_id(&self, id: &str) -> AppResult<Option<ExpenseCategoryRow>> {
        sqlx::query_as(include_str!(
            "../../queries/expense_categories/find_by_id.sql"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }
    pub async fn insert(&self, v: &ExpenseCategoryUpsertRequest) -> AppResult<ExpenseCategoryRow> {
        sqlx::query_as(include_str!("../../queries/expense_categories/insert.sql"))
            .bind(&v.name)
            .bind(&v.description)
            .bind(&v.parent_category_id)
            .fetch_one(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn update(
        &self,
        id: &str,
        v: &ExpenseCategoryUpsertRequest,
    ) -> AppResult<Option<ExpenseCategoryRow>> {
        sqlx::query_as(include_str!("../../queries/expense_categories/update.sql"))
            .bind(&v.name)
            .bind(&v.description)
            .bind(&v.parent_category_id)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn has_children(&self, id: &str) -> AppResult<bool> {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM expense_categories WHERE parent_category_id = ?)",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }
    pub async fn delete(&self, id: &str) -> AppResult<bool> {
        Ok(sqlx::query(include_str!(
            "../../queries/expense_categories/delete_by_id.sql"
        ))
        .bind(id)
        .execute(&self.pool)
        .await?
        .rows_affected()
            > 0)
    }
}
