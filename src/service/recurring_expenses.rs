use crate::{
    model::recurring_expenses::{RecurringExpense, RecurringExpenseUpsertRequest},
    repository::recurring_expenses::RecurringExpenseRepository,
    utils::error::{AppError, AppResult},
};
#[derive(Clone)]
pub struct RecurringExpenseService {
    repository: RecurringExpenseRepository,
}
impl RecurringExpenseService {
    pub fn new(repository: RecurringExpenseRepository) -> Self {
        Self { repository }
    }
    pub async fn list(&self) -> AppResult<Vec<RecurringExpense>> {
        Ok(self
            .repository
            .find_all()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }
    pub async fn get(&self, id: &str) -> AppResult<RecurringExpense> {
        self.repository
            .find_by_id(id)
            .await?
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("recurring expense", id))
    }
    pub async fn create(&self, v: &RecurringExpenseUpsertRequest) -> AppResult<RecurringExpense> {
        validate(v)?;
        self.repository.insert(v).await.map(Into::into)
    }
    pub async fn update(
        &self,
        id: &str,
        v: &RecurringExpenseUpsertRequest,
    ) -> AppResult<RecurringExpense> {
        validate(v)?;
        self.repository
            .update(id, v)
            .await?
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("recurring expense", id))
    }
    pub async fn delete(&self, id: &str) -> AppResult<()> {
        if self.repository.delete(id).await? {
            Ok(())
        } else {
            Err(AppError::not_found("recurring expense", id))
        }
    }
}
fn validate(v: &RecurringExpenseUpsertRequest) -> AppResult<()> {
    if v.name.trim().is_empty()
        || v.amount.trim().is_empty()
        || v.start_date.trim().is_empty()
        || v.category_id.trim().is_empty()
        || v.payment_method_id.trim().is_empty()
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
