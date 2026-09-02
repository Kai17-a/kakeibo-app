use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use std::collections::HashSet;

use crate::{
    database::models::income_categories::IncomeCategoryRow,
    model::income_categories::{
        IncomeCategoryQuery, IncomeCategorySortBy, IncomeCategorySortOrder,
        IncomeCategoryUpsertRequest,
    },
    utils::error::{AppError, AppResult},
};

#[derive(Clone)]
pub struct IncomeCategoryRepository {
    pool: SqlitePool,
}

impl IncomeCategoryRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_all(&self, query: &IncomeCategoryQuery) -> AppResult<Vec<IncomeCategoryRow>> {
        let mut builder = QueryBuilder::<Sqlite>::new(
            "SELECT category.id, category.created_at, category.updated_at, category.name, category.description, category.parent_category_id, category.display_order \
             FROM income_categories AS category LEFT JOIN income_categories AS parent ON parent.id = category.parent_category_id WHERE 1 = 1",
        );
        if let Some(id) = &query.id {
            builder.push(" AND category.id = ").push_bind(id);
        }

        let sort_by = query.sort_by.unwrap_or(IncomeCategorySortBy::Name);
        let sort_order = query.sort_order.unwrap_or(IncomeCategorySortOrder::Asc);
        let offset = u64::from(query.page() - 1) * u64::from(query.per_page());
        builder.push(" ORDER BY ");
        if matches!(sort_by, IncomeCategorySortBy::DisplayOrder) {
            builder
                .push("COALESCE(parent.display_order, category.display_order) ")
                .push(sort_order.sql())
                .push(", (category.parent_category_id IS NOT NULL) ASC")
                .push(", category.display_order ")
                .push(sort_order.sql());
        } else {
            builder
                .push("category.")
                .push(sort_by.sql())
                .push(" ")
                .push(sort_order.sql());
        }
        builder
            .push(", category.id ASC LIMIT ")
            .push_bind(i64::from(query.per_page()))
            .push(" OFFSET ")
            .push_bind(offset as i64);

        builder
            .build_query_as()
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn reorder(&self, parent_id: Option<&str>, ids: &[String]) -> AppResult<()> {
        let mut tx = self.pool.begin().await?;
        let siblings: Vec<String> =
            sqlx::query_scalar("SELECT id FROM income_categories WHERE parent_category_id IS ?")
                .bind(parent_id)
                .fetch_all(&mut *tx)
                .await?;
        validate_reorder(&siblings, ids)?;
        for (display_order, id) in ids.iter().enumerate() {
            sqlx::query("UPDATE income_categories SET display_order = ?, updated_at = current_timestamp WHERE id = ? AND parent_category_id IS ?")
                .bind(display_order as i64)
                .bind(id)
                .bind(parent_id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: &str) -> AppResult<Option<IncomeCategoryRow>> {
        sqlx::query_as(include_str!(
            "../../queries/income_categories/find_by_id.sql"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn insert(
        &self,
        input: &IncomeCategoryUpsertRequest,
    ) -> AppResult<IncomeCategoryRow> {
        sqlx::query_as(include_str!("../../queries/income_categories/insert.sql"))
            .bind(&input.name)
            .bind(&input.description)
            .bind(&input.parent_category_id)
            .fetch_one(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn update(
        &self,
        id: &str,
        input: &IncomeCategoryUpsertRequest,
    ) -> AppResult<Option<IncomeCategoryRow>> {
        sqlx::query_as(include_str!("../../queries/income_categories/update.sql"))
            .bind(&input.name)
            .bind(&input.description)
            .bind(&input.parent_category_id)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn has_children(&self, id: &str) -> AppResult<bool> {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM income_categories WHERE parent_category_id = ?)",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn delete(&self, id: &str) -> AppResult<bool> {
        let result = sqlx::query(include_str!(
            "../../queries/income_categories/delete_by_id.sql"
        ))
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
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
