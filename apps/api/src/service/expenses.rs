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
