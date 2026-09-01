use crate::{
    model::expenses::{Expense, ExpenseUpsertRequest},
    repository::expenses::{BudgetCrossing, ExpenseRepository},
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
        // バッチ基盤がないため、一覧取得のタイミングで当月分の定期支出を明細へ計上する。
        // insert_recurring_for_month は同月分が既にある場合は何もしない（冪等）。
        let month = self.repository.current_month().await?;
        self.repository.insert_recurring_for_month(&month).await?;
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
    pub async fn create(
        &self,
        v: &ExpenseUpsertRequest,
    ) -> AppResult<(Expense, Option<BudgetCrossing>)> {
        validate(v)?;
        let result = self.repository.insert_with_budget_check(v).await?;
        Ok((result.expense.into(), result.budget_crossing))
    }
    pub async fn update(
        &self,
        id: &str,
        v: &ExpenseUpsertRequest,
    ) -> AppResult<(Expense, Option<BudgetCrossing>)> {
        validate(v)?;
        self.repository
            .update_with_budget_check(id, v)
            .await?
            .map(|result| (result.expense.into(), result.budget_crossing))
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
    } else if v.amount.parse::<u64>().is_err() {
        Err(AppError::bad_request(
            "amount must be a non-negative integer",
        ))
    } else if v.amount.parse::<i64>().is_err() {
        Err(AppError::bad_request("amount must be within i64 range"))
    } else {
        Ok(())
    }
}
