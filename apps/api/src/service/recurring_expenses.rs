use crate::{
    model::recurring_expenses::{
        ExchangeRatePreview, RecurringExpense, RecurringExpenseUpsertRequest,
    },
    repository::recurring_expenses::RecurringExpenseRepository,
    service::exchange_rate::ExchangeRateService,
    utils::error::{AppError, AppResult},
};
#[derive(Clone)]
pub struct RecurringExpenseService {
    repository: RecurringExpenseRepository,
    exchange_rate: ExchangeRateService,
}
impl RecurringExpenseService {
    pub fn new(repository: RecurringExpenseRepository, exchange_rate: ExchangeRateService) -> Self {
        Self {
            repository,
            exchange_rate,
        }
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

    pub async fn exchange_rate_preview(
        &self,
        id: &str,
        month: &str,
    ) -> AppResult<ExchangeRatePreview> {
        let recurring = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("recurring expense", id))?;
        if recurring.currency_code.as_deref() != Some("USD") {
            return Err(AppError::bad_request("この固定費は外貨建てではない"));
        }
        let foreign_amount = recurring
            .foreign_amount
            .ok_or_else(|| AppError::bad_request("外貨建て固定費の外貨金額が設定されていません"))?;
        let foreign = foreign_amount
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite() && *value > 0.0)
            .ok_or_else(|| AppError::bad_request("foreign_amount must be a positive number"))?;
        let quote = self.exchange_rate.resolve(month).await.map_err(|error| {
            AppError::unprocessable_entity(&format!("為替レートを解決できません: {error}"))
        })?;
        let converted = foreign * quote.rate;
        if !converted.is_finite() || converted <= 0.0 {
            return Err(AppError::unprocessable_entity("換算額を計算できません"));
        }
        Ok(ExchangeRatePreview {
            foreign_amount,
            currency_code: "USD".to_owned(),
            exchange_rate: quote.rate.to_string(),
            exchange_rate_date: quote.effective_date,
            converted_amount: converted.round().to_string(),
        })
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
