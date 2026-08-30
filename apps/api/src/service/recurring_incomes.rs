use crate::{
    model::recurring_incomes::{RecurringIncome, RecurringIncomeUpsertRequest},
    repository::recurring_incomes::RecurringIncomeRepository,
    utils::error::{AppError, AppResult},
};

#[derive(Clone)]
pub struct RecurringIncomeService {
    repository: RecurringIncomeRepository,
}

impl RecurringIncomeService {
    pub fn new(repository: RecurringIncomeRepository) -> Self {
        Self { repository }
    }
    pub async fn list(&self) -> AppResult<Vec<RecurringIncome>> {
        Ok(self
            .repository
            .find_all()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }
    pub async fn get(&self, id: &str) -> AppResult<RecurringIncome> {
        self.repository
            .find_by_id(id)
            .await?
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("recurring income", id))
    }
    pub async fn create(&self, v: &RecurringIncomeUpsertRequest) -> AppResult<RecurringIncome> {
        validate(v)?;
        self.repository.insert(v).await.map(Into::into)
    }
    pub async fn update(
        &self,
        id: &str,
        v: &RecurringIncomeUpsertRequest,
    ) -> AppResult<RecurringIncome> {
        validate(v)?;
        self.repository
            .update(id, v)
            .await?
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("recurring income", id))
    }
    pub async fn delete(&self, id: &str) -> AppResult<()> {
        if self.repository.delete(id).await? {
            Ok(())
        } else {
            Err(AppError::not_found("recurring income", id))
        }
    }
}

fn validate(v: &RecurringIncomeUpsertRequest) -> AppResult<()> {
    if v.name.trim().is_empty()
        || v.amount.trim().is_empty()
        || v.start_date.trim().is_empty()
        || v.category_id.trim().is_empty()
    {
        return Err(AppError::bad_request("required fields must not be empty"));
    }
    if !(1..=31).contains(&v.payment_day) {
        return Err(AppError::bad_request(
            "payment_day must be between 1 and 31",
        ));
    }
    Ok(())
}
