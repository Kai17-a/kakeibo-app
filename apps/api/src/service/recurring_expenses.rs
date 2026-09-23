use crate::{
    database::models::recurring_expenses::RecurringExpenseRow,
    model::recurring_expenses::{
        BackfillResponse, ExchangeRatePreview, PendingMonthsResponse, RecurringExpense,
        RecurringExpenseUpsertRequest,
    },
    repository::expenses::ExpenseRepository,
    repository::recurring_expenses::RecurringExpenseRepository,
    service::exchange_rate::ExchangeRateService,
    service::expenses::recurring_conversion,
    utils::error::{AppError, AppResult},
};
#[derive(Clone)]
pub struct RecurringExpenseService {
    repository: RecurringExpenseRepository,
    expenses: ExpenseRepository,
    exchange_rate: ExchangeRateService,
}
impl RecurringExpenseService {
    pub fn new(
        repository: RecurringExpenseRepository,
        expenses: ExpenseRepository,
        exchange_rate: ExchangeRateService,
    ) -> Self {
        Self {
            repository,
            expenses,
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

    pub async fn pending_months(&self, id: &str) -> AppResult<PendingMonthsResponse> {
        let recurring = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("recurring expense", id))?;
        let current_month = self.expenses.current_month().await?;
        let recorded = self.expenses.find_recurring_expense_months(id).await?;
        Ok(PendingMonthsResponse {
            months: pending_months(&recurring, &current_month, &recorded)?,
        })
    }

    pub async fn backfill(&self, id: &str) -> AppResult<BackfillResponse> {
        let recurring = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("recurring expense", id))?;
        if recurring.is_variable {
            return Err(AppError::bad_request("準固定費は個別に登録してください"));
        }
        let current_month = self.expenses.current_month().await?;
        let recorded = self.expenses.find_recurring_expense_months(id).await?;
        let months = pending_months(&recurring, &current_month, &recorded)?;
        let mut response = BackfillResponse {
            created: Vec::new(),
            skipped: Vec::new(),
        };
        for month in months {
            let conversion = recurring_conversion(&self.exchange_rate, &month, &recurring).await;
            if recurring.currency_code.as_deref() == Some("USD") && conversion.is_none() {
                response.skipped.push(month);
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
            if self
                .expenses
                .insert_recurring(&month, &recurring, conversion_refs)
                .await?
            {
                response.created.push(month);
            }
        }
        Ok(response)
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

fn pending_months(
    recurring: &RecurringExpenseRow,
    current_month: &str,
    recorded: &[String],
) -> AppResult<Vec<String>> {
    if !recurring.is_active {
        return Ok(Vec::new());
    }
    let start = parse_month(&recurring.start_date)?;
    let previous = previous_month(parse_month(current_month)?);
    let end = recurring
        .end_date
        .as_deref()
        .map(parse_month)
        .transpose()?
        .map_or(previous, |month| month.min(previous));
    let recorded = recorded.iter().collect::<std::collections::HashSet<_>>();
    let mut result = Vec::new();
    let mut month = start;
    while month <= end {
        let formatted = format_month(month);
        if !recorded.contains(&formatted) {
            result.push(formatted);
        }
        month = next_month(month);
    }
    Ok(result)
}

fn parse_month(value: &str) -> AppResult<(i32, u32)> {
    let month = value
        .get(..7)
        .ok_or_else(|| AppError::bad_request("date must be valid"))?;
    let (year, month) = month
        .split_once('-')
        .and_then(|(year, month)| Some((year.parse().ok()?, month.parse().ok()?)))
        .ok_or_else(|| AppError::bad_request("date must be valid"))?;
    if !(1..=12).contains(&month) {
        return Err(AppError::bad_request("date must be valid"));
    }
    Ok((year, month))
}

fn format_month((year, month): (i32, u32)) -> String {
    format!("{year:04}-{month:02}")
}

fn next_month((year, month): (i32, u32)) -> (i32, u32) {
    if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    }
}

fn previous_month((year, month): (i32, u32)) -> (i32, u32) {
    if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
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
