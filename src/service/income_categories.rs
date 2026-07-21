use crate::{
    model::income_categories::{
        IncomeCategory, IncomeCategoryListResponse, IncomeCategoryPagination, IncomeCategoryQuery,
        IncomeCategoryUpsertRequest,
    },
    repository::income_categories::IncomeCategoryRepository,
    utils::error::{AppError, AppResult},
};

#[derive(Clone)]
pub struct IncomeCategoryService {
    repository: IncomeCategoryRepository,
}

impl IncomeCategoryService {
    pub fn new(repository: IncomeCategoryRepository) -> Self {
        Self { repository }
    }

    pub async fn list(&self, query: &IncomeCategoryQuery) -> AppResult<IncomeCategoryListResponse> {
        validate_query(query)?;
        let items = self
            .repository
            .find_all(query)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();
        Ok(IncomeCategoryListResponse {
            items,
            pagination: IncomeCategoryPagination {
                page: query.page(),
                per_page: query.per_page(),
            },
        })
    }

    pub async fn get(&self, id: &str) -> AppResult<IncomeCategory> {
        self.repository
            .find_by_id(id)
            .await?
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("income category", id))
    }

    pub async fn create(&self, input: &IncomeCategoryUpsertRequest) -> AppResult<IncomeCategory> {
        validate(input)?;
        self.repository.insert(input).await.map(Into::into)
    }

    pub async fn update(
        &self,
        id: &str,
        input: &IncomeCategoryUpsertRequest,
    ) -> AppResult<IncomeCategory> {
        validate(input)?;
        self.repository
            .update(id, input)
            .await?
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("income category", id))
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        if self.repository.delete(id).await? {
            Ok(())
        } else {
            Err(AppError::not_found("income category", id))
        }
    }
}

fn validate_query(query: &IncomeCategoryQuery) -> AppResult<()> {
    if query.page == Some(0) {
        return Err(AppError::bad_request("page must be greater than zero"));
    }
    if query
        .per_page
        .is_some_and(|value| value == 0 || value > 100)
    {
        return Err(AppError::bad_request("per_page must be between 1 and 100"));
    }
    Ok(())
}

fn validate(input: &IncomeCategoryUpsertRequest) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::bad_request("name is required"));
    }
    Ok(())
}
