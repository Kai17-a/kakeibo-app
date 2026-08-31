use crate::{
    model::payment_methods::{
        PaymentMethod, PaymentMethodListResponse, PaymentMethodPagination, PaymentMethodQuery,
        PaymentMethodUpsertRequest,
    },
    repository::payment_methods::PaymentMethodRepository,
    utils::error::{AppError, AppResult},
};
#[derive(Clone)]
pub struct PaymentMethodService {
    repository: PaymentMethodRepository,
}
impl PaymentMethodService {
    pub fn new(repository: PaymentMethodRepository) -> Self {
        Self { repository }
    }
    pub async fn list(&self, q: &PaymentMethodQuery) -> AppResult<PaymentMethodListResponse> {
        if q.page == Some(0) || q.per_page.is_some_and(|v| v == 0 || v > 100) {
            return Err(AppError::bad_request("invalid pagination"));
        }
        Ok(PaymentMethodListResponse {
            items: self
                .repository
                .find_all(q)
                .await?
                .into_iter()
                .map(TryInto::try_into)
                .collect::<AppResult<Vec<_>>>()?,
            pagination: PaymentMethodPagination {
                page: q.page(),
                per_page: q.per_page(),
            },
        })
    }
    pub async fn get(&self, id: &str) -> AppResult<PaymentMethod> {
        self.repository
            .find_by_id(id)
            .await?
            .map(TryInto::try_into)
            .transpose()?
            .ok_or_else(|| AppError::not_found("payment method", id))
    }
    pub async fn create(&self, v: &PaymentMethodUpsertRequest) -> AppResult<PaymentMethod> {
        validate(v)?;
        self.repository.insert(v).await?.try_into()
    }
    pub async fn update(
        &self,
        id: &str,
        v: &PaymentMethodUpsertRequest,
    ) -> AppResult<PaymentMethod> {
        validate(v)?;
        self.repository
            .update(id, v)
            .await?
            .map(TryInto::try_into)
            .transpose()?
            .ok_or_else(|| AppError::not_found("payment method", id))
    }
    pub async fn delete(&self, id: &str) -> AppResult<()> {
        if self.repository.is_referenced_by_transactions(id).await? {
            return Err(AppError::bad_request(
                "payment method is used by an expense or income and cannot be deleted",
            ));
        }
        if self.repository.delete(id).await? {
            Ok(())
        } else {
            Err(AppError::not_found("payment method", id))
        }
    }
}
fn validate(v: &PaymentMethodUpsertRequest) -> AppResult<()> {
    if v.name.trim().is_empty() {
        return Err(AppError::bad_request("name is required"));
    }
    if let Some(initial_balance) = &v.initial_balance {
        if initial_balance.is_empty() || initial_balance.parse::<u64>().is_err() {
            return Err(AppError::bad_request(
                "initial_balance must be a non-negative integer",
            ));
        }
        if initial_balance.parse::<i64>().is_err() {
            return Err(AppError::bad_request(
                "initial_balance must be within i64 range",
            ));
        }
    }
    Ok(())
}
