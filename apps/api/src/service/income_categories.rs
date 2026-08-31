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
        self.validate_parent(None, input.parent_category_id.as_deref())
            .await?;
        self.repository.insert(input).await.map(Into::into)
    }

    pub async fn update(
        &self,
        id: &str,
        input: &IncomeCategoryUpsertRequest,
    ) -> AppResult<IncomeCategory> {
        validate(input)?;
        self.validate_parent(Some(id), input.parent_category_id.as_deref())
            .await?;
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

    async fn validate_parent(&self, id: Option<&str>, parent_id: Option<&str>) -> AppResult<()> {
        let Some(parent_id) = parent_id else {
            return Ok(());
        };
        if id == Some(parent_id) {
            return Err(AppError::bad_request("a category cannot be its own parent"));
        }
        let parent = self
            .repository
            .find_by_id(parent_id)
            .await?
            .ok_or_else(|| AppError::bad_request("parent category does not exist"))?;
        if parent.parent_category_id.is_some() {
            return Err(AppError::bad_request("a child category cannot be a parent"));
        }
        if let Some(id) = id
            && self.repository.has_children(id).await?
        {
            return Err(AppError::bad_request(
                "a category with children cannot become a child",
            ));
        }
        Ok(())
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
