use crate::{
    database::models::expense_categories::ExpenseCategoryRow,
    model::expense_categories::{
        ExpenseCategoryQuery, ExpenseCategorySortBy, ExpenseCategorySortOrder,
        ExpenseCategoryUpsertRequest,
    },
    utils::error::{AppError, AppResult},
};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use std::collections::HashSet;
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
            "SELECT category.id, category.created_at, category.updated_at, category.name, category.description, category.parent_category_id, category.display_order FROM expense_categories AS category LEFT JOIN expense_categories AS parent ON parent.id = category.parent_category_id WHERE 1=1",
        );
        if let Some(id) = &q.id {
            b.push(" AND category.id = ").push_bind(id);
        }
        let sort = q.sort_by.unwrap_or(ExpenseCategorySortBy::Name);
        let order = q.sort_order.unwrap_or(ExpenseCategorySortOrder::Asc);
        b.push(" ORDER BY ");
        if matches!(sort, ExpenseCategorySortBy::DisplayOrder) {
            b.push("COALESCE(parent.display_order, category.display_order) ")
                .push(order.sql())
                .push(", (category.parent_category_id IS NOT NULL) ASC")
                .push(", category.display_order ")
                .push(order.sql());
        } else {
            b.push("category.")
                .push(sort.sql())
                .push(" ")
                .push(order.sql());
        }
        b.push(", category.id ASC LIMIT ")
            .push_bind(i64::from(q.per_page()))
            .push(" OFFSET ")
            .push_bind((u64::from(q.page() - 1) * u64::from(q.per_page())) as i64);
        b.build_query_as()
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn reorder(&self, parent_id: Option<&str>, ids: &[String]) -> AppResult<()> {
        let mut tx = self.pool.begin().await?;
        let siblings: Vec<String> =
            sqlx::query_scalar("SELECT id FROM expense_categories WHERE parent_category_id IS ?")
                .bind(parent_id)
                .fetch_all(&mut *tx)
                .await?;
        validate_reorder(&siblings, ids)?;
        for (display_order, id) in ids.iter().enumerate() {
            sqlx::query("UPDATE expense_categories SET display_order = ?, updated_at = current_timestamp WHERE id = ? AND parent_category_id IS ?")
                .bind(display_order as i64)
                .bind(id)
                .bind(parent_id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
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
    pub async fn is_used(&self, id: &str) -> AppResult<bool> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM expenses WHERE category_id = ?1 UNION ALL SELECT 1 FROM budgets WHERE category_id = ?1)")
            .bind(id).fetch_one(&self.pool).await.map_err(Into::into)
    }
}

fn validate_reorder(siblings: &[String], requested: &[String]) -> AppResult<()> {
    let sibling_set: HashSet<_> = siblings.iter().collect();
    let requested_set: HashSet<_> = requested.iter().collect();
    if requested_set.len() != requested.len() {
        return Err(AppError::bad_request(
            "category_ids must not contain duplicates",
        ));
    }
    if sibling_set != requested_set {
        return Err(AppError::bad_request(
            "category_ids must exactly match all categories in the requested sibling scope",
        ));
    }
    Ok(())
}
