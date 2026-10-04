use crate::{
    model::transfers::{Transfer, TransferUpsertRequest},
    repository::transfers::TransferRepository,
    utils::error::{AppError, AppResult},
};
use chrono::NaiveDate;

#[derive(Clone)]
pub struct TransferService {
    repository: TransferRepository,
}

impl TransferService {
    pub fn new(repository: TransferRepository) -> Self {
        Self { repository }
    }
    pub async fn list(&self) -> AppResult<Vec<Transfer>> {
        Ok(self
            .repository
            .post_recurring_and_find_all()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }
    pub async fn get(&self, id: &str) -> AppResult<Transfer> {
        self.repository
            .find_by_id(id)
            .await?
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("transfer", id))
    }
    pub async fn create(&self, value: &TransferUpsertRequest) -> AppResult<Transfer> {
        self.validate(value).await?;
        self.repository.insert(value).await.map(Into::into)
    }
    pub async fn update(&self, id: &str, value: &TransferUpsertRequest) -> AppResult<Transfer> {
        self.validate(value).await?;
        self.repository
            .update(id, value)
            .await?
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("transfer", id))
    }
    pub async fn delete(&self, id: &str) -> AppResult<()> {
        if self.repository.delete(id).await? {
            Ok(())
        } else {
            Err(AppError::not_found("transfer", id))
        }
    }
    async fn validate(&self, value: &TransferUpsertRequest) -> AppResult<()> {
        if value.transaction_date != value.transaction_date.trim()
            || NaiveDate::parse_from_str(&value.transaction_date, "%Y-%m-%d").is_err()
        {
            return Err(AppError::bad_request("transaction_date must be YYYY-MM-DD"));
        }
        if value.amount != value.amount.trim() || !is_valid_amount(&value.amount) {
            return Err(AppError::bad_request(
                "amount must be a positive integer within i64 range",
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

pub(crate) fn is_valid_amount(value: &str) -> bool {
    matches!(value.parse::<u64>(), Ok(amount) if amount >= 1) && value.parse::<i64>().is_ok()
}
