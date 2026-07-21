use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::{
    database::models::income_categories::IncomeCategoryRow,
    model::income_categories::{
        IncomeCategoryQuery, IncomeCategorySortBy, IncomeCategorySortOrder,
        IncomeCategoryUpsertRequest,
    },
    utils::error::AppResult,
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
            "SELECT id, created_at, updated_at, name, description \
             FROM income_categories WHERE 1 = 1",
        );
        if let Some(id) = &query.id {
            builder.push(" AND id = ").push_bind(id);
        }

        let sort_by = query.sort_by.unwrap_or(IncomeCategorySortBy::Name);
        let sort_order = query.sort_order.unwrap_or(IncomeCategorySortOrder::Asc);
        let offset = u64::from(query.page() - 1) * u64::from(query.per_page());
        builder
            .push(" ORDER BY ")
            .push(sort_by.sql())
            .push(" ")
            .push(sort_order.sql())
            .push(", id ASC LIMIT ")
            .push_bind(i64::from(query.per_page()))
            .push(" OFFSET ")
            .push_bind(offset as i64);

        builder
            .build_query_as()
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
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
            .bind(id)
            .fetch_optional(&self.pool)
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
