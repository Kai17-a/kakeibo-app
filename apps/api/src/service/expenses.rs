use crate::{
    model::expenses::{Expense, ExpenseUpsertRequest},
    repository::expenses::ExpenseRepository,
    utils::error::{AppError, AppResult},
};
#[derive(Clone)]
pub struct ExpenseService {
    repository: ExpenseRepository,
}
impl ExpenseService {
    pub fn new(repository: ExpenseRepository) -> Self {
        Self { repository }
    }
    pub async fn list(&self) -> AppResult<Vec<Expense>> {
        Ok(self
            .repository
            .find_all()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }
    pub async fn get(&self, id: &str) -> AppResult<Expense> {
        self.repository
            .find_by_id(id)
            .await?
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("expense", id))
    }
    pub async fn create(&self, v: &ExpenseUpsertRequest) -> AppResult<Expense> {
        validate(v)?;
        self.repository.insert(v).await.map(Into::into)
    }
    pub async fn update(&self, id: &str, v: &ExpenseUpsertRequest) -> AppResult<Expense> {
        validate(v)?;
        self.repository
            .update(id, v)
            .await?
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("expense", id))
    }
    pub async fn delete(&self, id: &str) -> AppResult<()> {
        if self.repository.delete(id).await? {
            Ok(())
        } else {
            Err(AppError::not_found("expense", id))
        }
    }
}
fn validate(v: &ExpenseUpsertRequest) -> AppResult<()> {
    if v.transaction_date.trim().is_empty()
        || v.amount.trim().is_empty()
        || v.category_id.trim().is_empty()
        || v.payment_method_id.trim().is_empty()
    {
        Err(AppError::bad_request("required fields must not be empty"))
    } else {
        Ok(())
    }
}
