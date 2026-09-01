use crate::{
    model::budgets::{Budget, BudgetUpsertRequest},
    repository::budgets::BudgetRepository,
    utils::error::{AppError, AppResult},
};

#[derive(Clone)]
pub struct BudgetService {
    repository: BudgetRepository,
}
impl BudgetService {
    pub fn new(repository: BudgetRepository) -> Self {
        Self { repository }
    }
    pub async fn list(&self) -> AppResult<Vec<Budget>> {
        Ok(self
            .repository
            .find_all()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }
    pub async fn get(&self, id: &str) -> AppResult<Budget> {
        self.repository
            .find_by_id(id)
            .await?
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("budget", id))
    }
    pub async fn create(&self, v: &BudgetUpsertRequest) -> AppResult<Budget> {
        self.validate(v).await?;
        self.repository.insert(v).await.map(Into::into)
    }
    pub async fn update(&self, id: &str, v: &BudgetUpsertRequest) -> AppResult<Budget> {
        self.validate(v).await?;
        self.repository
            .update(id, v)
            .await?
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("budget", id))
    }
    pub async fn delete(&self, id: &str) -> AppResult<()> {
        if self.repository.delete(id).await? {
            Ok(())
        } else {
            Err(AppError::not_found("budget", id))
        }
    }
    async fn validate(&self, v: &BudgetUpsertRequest) -> AppResult<()> {
        if v.category_id.trim().is_empty() || v.amount.trim().is_empty() {
            return Err(AppError::bad_request("required fields must not be empty"));
        }
        let amount = v
            .amount
            .parse::<u64>()
            .map_err(|_| AppError::bad_request("amount must be a non-negative integer"))?;
        if amount > i64::MAX as u64 {
            return Err(AppError::bad_request(
                "amount must be within the signed 64-bit integer range",
            ));
        }
        if !self.repository.category_exists(&v.category_id).await? {
            return Err(AppError::bad_request("expense category does not exist"));
        }
        Ok(())
    }
}
