use crate::{
    model::expense_categories::{
        ExpenseCategory, ExpenseCategoryListResponse, ExpenseCategoryPagination,
        ExpenseCategoryQuery, ExpenseCategoryUpsertRequest,
    },
    repository::expense_categories::ExpenseCategoryRepository,
    utils::error::{AppError, AppResult},
};
#[derive(Clone)]
pub struct ExpenseCategoryService {
    repository: ExpenseCategoryRepository,
}
impl ExpenseCategoryService {
    pub fn new(repository: ExpenseCategoryRepository) -> Self {
        Self { repository }
    }
    pub async fn list(&self, q: &ExpenseCategoryQuery) -> AppResult<ExpenseCategoryListResponse> {
        validate_query(q)?;
        Ok(ExpenseCategoryListResponse {
            items: self
                .repository
                .find_all(q)
                .await?
                .into_iter()
                .map(Into::into)
                .collect(),
            pagination: ExpenseCategoryPagination {
                page: q.page(),
                per_page: q.per_page(),
            },
        })
    }
    pub async fn get(&self, id: &str) -> AppResult<ExpenseCategory> {
        self.repository
            .find_by_id(id)
            .await?
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("expense category", id))
    }
    pub async fn create(&self, v: &ExpenseCategoryUpsertRequest) -> AppResult<ExpenseCategory> {
        validate(v)?;
        self.repository.insert(v).await.map(Into::into)
    }
    pub async fn update(
        &self,
        id: &str,
        v: &ExpenseCategoryUpsertRequest,
    ) -> AppResult<ExpenseCategory> {
        validate(v)?;
        self.repository
            .update(id, v)
            .await?
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("expense category", id))
    }
    pub async fn delete(&self, id: &str) -> AppResult<()> {
        if self.repository.delete(id).await? {
            Ok(())
        } else {
            Err(AppError::not_found("expense category", id))
        }
    }
}
fn validate_query(q: &ExpenseCategoryQuery) -> AppResult<()> {
    if q.page == Some(0) {
        return Err(AppError::bad_request("page must be greater than zero"));
    }
    if q.per_page.is_some_and(|v| v == 0 || v > 100) {
        return Err(AppError::bad_request("per_page must be between 1 and 100"));
    }
    Ok(())
}
fn validate(v: &ExpenseCategoryUpsertRequest) -> AppResult<()> {
    if v.name.trim().is_empty() {
        Err(AppError::bad_request("name is required"))
    } else {
        Ok(())
    }
}
