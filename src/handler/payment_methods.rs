use crate::{
    model::payment_methods::{
        PaymentMethod, PaymentMethodListResponse, PaymentMethodQuery, PaymentMethodUpsertRequest,
    },
    service::payment_methods::PaymentMethodService,
    utils::error::{AppError, AppResult},
};
use axum::{
    Json,
    extract::{Path, Query, State, rejection::QueryRejection},
    http::StatusCode,
};
#[derive(Clone)]
pub struct AppState {
    pub payment_methods: PaymentMethodService,
}
#[utoipa::path(get,path="/api/payment-methods",params(PaymentMethodQuery),responses((status=200,body=PaymentMethodListResponse),(status=400)))]
pub async fn list(
    State(s): State<AppState>,
    q: Result<Query<PaymentMethodQuery>, QueryRejection>,
) -> AppResult<Json<PaymentMethodListResponse>> {
    let Query(q) =
        q.map_err(|e| AppError::bad_request(&format!("Invalid query parameters: {e}")))?;
    Ok(Json(s.payment_methods.list(&q).await?))
}
#[utoipa::path(get,path="/api/payment-methods/{id}",params(("id"=String,Path)),responses((status=200,body=PaymentMethod),(status=404)))]
pub async fn get(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<PaymentMethod>> {
    Ok(Json(s.payment_methods.get(&id).await?))
}
#[utoipa::path(post,path="/api/payment-methods",request_body=PaymentMethodUpsertRequest,responses((status=201,body=PaymentMethod),(status=400)))]
pub async fn create(
    State(s): State<AppState>,
    Json(v): Json<PaymentMethodUpsertRequest>,
) -> AppResult<(StatusCode, Json<PaymentMethod>)> {
    Ok((
        StatusCode::CREATED,
        Json(s.payment_methods.create(&v).await?),
    ))
}
#[utoipa::path(put,path="/api/payment-methods/{id}",params(("id"=String,Path)),request_body=PaymentMethodUpsertRequest,responses((status=200,body=PaymentMethod),(status=400),(status=404)))]
pub async fn update(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(v): Json<PaymentMethodUpsertRequest>,
) -> AppResult<Json<PaymentMethod>> {
    Ok(Json(s.payment_methods.update(&id, &v).await?))
}
#[utoipa::path(delete,path="/api/payment-methods/{id}",params(("id"=String,Path)),responses((status=204),(status=404)))]
pub async fn delete(State(s): State<AppState>, Path(id): Path<String>) -> AppResult<StatusCode> {
    s.payment_methods.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
