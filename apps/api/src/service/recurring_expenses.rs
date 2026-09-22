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
        let v = normalized(v);
        self.repository.insert(&v).await.map(Into::into)
    }
    pub async fn update(
        &self,
        id: &str,
        v: &RecurringExpenseUpsertRequest,
    ) -> AppResult<RecurringExpense> {
        validate(v)?;
        let v = normalized(v);
        self.repository
            .update(id, &v)
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
    if let (Some(amount), Some(code)) = (&v.foreign_amount, &v.currency_code) {
        let valid_amount = amount.parse::<f64>().is_ok_and(|value| value > 0.0);
        let valid_rate = v
            .exchange_rate
            .as_ref()
            .is_none_or(|rate| rate.parse::<f64>().is_ok_and(|value| value > 0.0));
        if !valid_amount || !valid_rate || !code.eq_ignore_ascii_case("USD") {
            return Err(AppError::bad_request(
                "foreign_amount must be positive, currency_code must be USD, and exchange_rate must be positive when provided",
            ));
        }
    } else if v.foreign_amount.is_some() || v.currency_code.is_some() || v.exchange_rate.is_some() {
        return Err(AppError::bad_request(
            "foreign_amount, currency_code and exchange_rate must be provided together",
        ));
    }
    Ok(())
}

fn normalized(v: &RecurringExpenseUpsertRequest) -> RecurringExpenseUpsertRequest {
    let mut normalized = v.clone();
    normalized.currency_code = v
        .currency_code
        .as_ref()
        .map(|code| code.trim().to_ascii_uppercase());
    normalized
}
