use chrono::NaiveDate;

use crate::{
    model::incomes::{Income, IncomeListResponse, IncomeQuery, IncomeUpsertRequest, Pagination},
    repository::incomes::IncomeRepository,
    utils::error::{AppError, AppResult},
};

#[derive(Clone)]
pub struct IncomeService {
    repository: IncomeRepository,
}

impl IncomeService {
    pub fn new(repository: IncomeRepository) -> Self {
        Self { repository }
    }

    pub async fn list(&self, query: &IncomeQuery) -> AppResult<IncomeListResponse> {
        validate_query(query)?;
        let month = self.repository.current_month().await?;
        self.repository.insert_recurring_for_month(&month).await?;
        let items = self
            .repository
            .find_all(query)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();
        let page = query.page();
        let per_page = query.per_page();

        Ok(IncomeListResponse {
            items,
            pagination: Pagination { page, per_page },
        })
    }

    pub async fn get(&self, id: &str) -> AppResult<Income> {
        self.repository
            .find_by_id(id)
            .await?
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("income", id))
    }

    pub async fn create(&self, input: &IncomeUpsertRequest) -> AppResult<Income> {
        validate(input)?;
        self.repository.insert(input).await.map(Into::into)
    }

    pub async fn update(&self, id: &str, input: &IncomeUpsertRequest) -> AppResult<Income> {
        validate(input)?;
        self.repository
            .update(id, input)
            .await?
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("income", id))
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        if self.repository.delete(id).await? {
            Ok(())
        } else {
            Err(AppError::not_found("income", id))
        }
    }
}

fn validate_query(query: &IncomeQuery) -> AppResult<()> {
    let date_from = query.date_from.as_deref().map(parse_date).transpose()?;
    let date_to = query.date_to.as_deref().map(parse_date).transpose()?;

    if date_from.zip(date_to).is_some_and(|(from, to)| from > to) {
        return Err(AppError::bad_request(
            "date_from must be earlier than or equal to date_to",
        ));
    }
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

fn parse_date(value: &str) -> AppResult<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| AppError::bad_request("dates must use YYYY-MM-DD format"))
}

fn validate(input: &IncomeUpsertRequest) -> AppResult<()> {
    if input.category_id.trim().is_empty()
        || input.transaction_date.trim().is_empty()
        || input.amount.trim().is_empty()
    {
        return Err(AppError::bad_request(
            "category_id, transaction_date and amount are required",
        ));
    }
    Ok(())
}
