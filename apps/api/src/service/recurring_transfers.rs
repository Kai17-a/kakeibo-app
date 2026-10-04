use crate::{
    model::recurring_transfers::{RecurringTransfer, RecurringTransferUpsertRequest},
    repository::recurring_transfers::RecurringTransferRepository,
    service::transfers::is_valid_amount,
    utils::error::{AppError, AppResult},
};
use chrono::NaiveDate;

#[derive(Clone)]
pub struct RecurringTransferService {
    repository: RecurringTransferRepository,
}

impl RecurringTransferService {
    pub fn new(repository: RecurringTransferRepository) -> Self {
        Self { repository }
    }
    pub async fn list(&self) -> AppResult<Vec<RecurringTransfer>> {
        Ok(self
            .repository
            .find_all()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }
    pub async fn get(&self, id: &str) -> AppResult<RecurringTransfer> {
        self.repository
            .find_by_id(id)
            .await?
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("recurring transfer", id))
    }
    pub async fn create(
        &self,
        value: &RecurringTransferUpsertRequest,
    ) -> AppResult<RecurringTransfer> {
        self.validate(value).await?;
        self.repository.insert(value).await.map(Into::into)
    }
    pub async fn update(
        &self,
        id: &str,
        value: &RecurringTransferUpsertRequest,
    ) -> AppResult<RecurringTransfer> {
        self.validate(value).await?;
        self.repository
            .update(id, value)
            .await?
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("recurring transfer", id))
    }
    pub async fn delete(&self, id: &str) -> AppResult<()> {
        if self.repository.is_referenced_by_transfer(id).await? {
            return Err(AppError::conflict(
                "計上済みの振替明細があるため削除できません。停止する場合は無効にするか終了日を設定してください。",
            ));
        }
        if self.repository.delete(id).await? {
            Ok(())
        } else {
            Err(AppError::not_found("recurring transfer", id))
        }
    }
    async fn validate(&self, value: &RecurringTransferUpsertRequest) -> AppResult<()> {
        if value.name.trim().is_empty() {
            return Err(AppError::bad_request("name is required"));
        }
        if value.amount != value.amount.trim() || !is_valid_amount(&value.amount) {
            return Err(AppError::bad_request(
                "amount must be a positive integer within i64 range",
            ));
        }
        if !(1..=31).contains(&value.payment_day) {
            return Err(AppError::bad_request(
                "payment_day must be between 1 and 31",
            ));
        }
        let start = parse_date(&value.start_date)?;
        if value
            .end_date
            .as_deref()
            .map(parse_date)
            .transpose()?
            .is_some_and(|end| end < start)
        {
            return Err(AppError::bad_request(
                "end_date must be later than or equal to start_date",
            ));
        }
        if value.from_payment_method_id.trim().is_empty()
            || value.to_payment_method_id.trim().is_empty()
        {
            return Err(AppError::bad_request("payment methods are required"));
        }
        if value.from_payment_method_id == value.to_payment_method_id {
            return Err(AppError::bad_request(
                "source and destination payment methods must differ",
            ));
        }
        for id in [&value.from_payment_method_id, &value.to_payment_method_id] {
            if !self.repository.payment_method_exists(id).await? {
                return Err(AppError::bad_request("payment method does not exist"));
            }
        }
        Ok(())
    }
}

fn parse_date(value: &str) -> AppResult<NaiveDate> {
    if value != value.trim() {
        return Err(AppError::bad_request("dates must use YYYY-MM-DD format"));
    }
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| AppError::bad_request("dates must use YYYY-MM-DD format"))
}
