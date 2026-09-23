use crate::{
    model::expenses::{Expense, ExpenseUpsertRequest},
    repository::expenses::{BudgetCrossing, ExpenseRepository},
    service::exchange_rate::ExchangeRateService,
    utils::error::{AppError, AppResult},
};
#[derive(Clone)]
pub struct ExpenseService {
    repository: ExpenseRepository,
    exchange_rate: ExchangeRateService,
}
impl ExpenseService {
    pub fn new(repository: ExpenseRepository, exchange_rate: ExchangeRateService) -> Self {
        Self {
            repository,
            exchange_rate,
        }
    }
    pub async fn list(&self) -> AppResult<Vec<Expense>> {
        // バッチ基盤がないため、一覧取得のタイミングで当月分の定期支出を明細へ計上する。
        // 同月分が既にある定期支出は何もしない（冪等）。
        let month = self.repository.current_month().await?;
        for recurring in self.repository.find_recurring_for_month(&month).await? {
            let conversion = if recurring.currency_code.as_deref() == Some("USD") {
                match self.exchange_rate.resolve(&month).await {
                    Ok(quote) => match recurring
                        .foreign_amount
                        .as_deref()
                        .unwrap_or_default()
                        .parse::<f64>()
                    {
                        Ok(foreign)
                            if foreign.is_finite()
                                && foreign > 0.0
                                && quote.rate.is_finite()
                                && quote.rate > 0.0 =>
                        {
                            let converted = foreign * quote.rate;
                            if converted.is_finite() && converted > 0.0 {
                                Some((
                                    converted.round().to_string(),
                                    recurring.foreign_amount.clone().unwrap_or_default(),
                                    quote.rate.to_string(),
                                    quote.effective_date,
                                ))
                            } else {
                                tracing::error!(recurring_expense_id = %recurring.id, "Skipping recurring expense because converted USD amount overflowed");
                                None
                            }
                        }
                        _ => {
                            tracing::error!(recurring_expense_id = %recurring.id, "Skipping recurring expense with invalid USD amount");
                            None
                        }
                    },
                    Err(error) => {
                        tracing::error!(recurring_expense_id = %recurring.id, %error, "Skipping recurring USD expense because exchange rate resolution failed");
                        None
                    }
                }
            } else {
                None
            };
            if recurring.currency_code.as_deref() == Some("USD") && conversion.is_none() {
                continue;
            }
            let conversion_refs = conversion.as_ref().map(|(amount, foreign, rate, date)| {
                (
                    amount.as_str(),
                    foreign.as_str(),
                    rate.as_str(),
                    date.as_str(),
                )
            });
            self.repository
                .insert_recurring(&month, &recurring, conversion_refs)
                .await?;
        }
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
        let v = normalized(v);
        let result = self.repository.insert_with_budget_check(&v).await?;
        Ok((result.expense.into(), result.budget_crossing))
    }
    pub async fn update(
        &self,
        id: &str,
        v: &ExpenseUpsertRequest,
    ) -> AppResult<(Expense, Option<BudgetCrossing>)> {
        validate(v)?;
        let v = normalized(v);
        self.repository
            .update_with_budget_check(id, &v)
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
        validate_foreign_currency(v)
    }
}

fn validate_foreign_currency(v: &ExpenseUpsertRequest) -> AppResult<()> {
    let has_foreign_fields = v.foreign_amount.is_some()
        || v.currency_code.is_some()
        || v.exchange_rate.is_some()
        || v.exchange_rate_date.is_some();
    if !has_foreign_fields {
        return Ok(());
    }

    if !v
        .currency_code
        .as_deref()
        .is_some_and(|code| code.eq_ignore_ascii_case("USD"))
    {
        return Err(AppError::bad_request("currency_code must be USD"));
    }
    let foreign_amount = v.foreign_amount.as_deref().ok_or_else(|| {
        AppError::bad_request("foreign_amount is required when currency_code is USD")
    })?;
    let exchange_rate = v.exchange_rate.as_deref().ok_or_else(|| {
        AppError::bad_request("exchange_rate is required when currency_code is USD")
    })?;
    let exchange_rate_date = v.exchange_rate_date.as_deref().ok_or_else(|| {
        AppError::bad_request("exchange_rate_date is required when currency_code is USD")
    })?;
    if !foreign_amount
        .parse::<f64>()
        .is_ok_and(|value| value.is_finite() && value > 0.0)
        || !exchange_rate
            .parse::<f64>()
            .is_ok_and(|value| value.is_finite() && value > 0.0)
    {
        return Err(AppError::bad_request(
            "foreign_amount and exchange_rate must be positive numbers",
        ));
    }
    if exchange_rate_date.trim().is_empty() {
        return Err(AppError::bad_request(
            "exchange_rate_date must not be empty",
        ));
    }
    Ok(())
}

fn normalized(v: &ExpenseUpsertRequest) -> ExpenseUpsertRequest {
    let mut normalized = v.clone();
    normalized.currency_code = v
        .currency_code
        .as_ref()
        .map(|code| code.trim().to_ascii_uppercase());
    normalized
}
