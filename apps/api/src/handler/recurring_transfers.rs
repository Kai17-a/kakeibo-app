use crate::{
    model::recurring_transfers::{RecurringTransfer, RecurringTransferUpsertRequest},
    service::recurring_transfers::RecurringTransferService,
    utils::error::AppResult,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

#[derive(Clone)]
pub struct AppState {
    pub recurring_transfers: RecurringTransferService,
}

#[utoipa::path(get,path="/api/recurring-transfers",responses((status=200,body=Vec<RecurringTransfer>)))]
pub async fn list(State(state): State<AppState>) -> AppResult<Json<Vec<RecurringTransfer>>> {
    Ok(Json(state.recurring_transfers.list().await?))
}
#[utoipa::path(get,path="/api/recurring-transfers/{id}",params(("id"=String,Path)),responses((status=200,body=RecurringTransfer),(status=404)))]
pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<RecurringTransfer>> {
    Ok(Json(state.recurring_transfers.get(&id).await?))
}
#[utoipa::path(post,path="/api/recurring-transfers",request_body=RecurringTransferUpsertRequest,responses((status=201,body=RecurringTransfer),(status=400)))]
pub async fn create(
    State(state): State<AppState>,
    Json(value): Json<RecurringTransferUpsertRequest>,
) -> AppResult<(StatusCode, Json<RecurringTransfer>)> {
    Ok((
        StatusCode::CREATED,
        Json(state.recurring_transfers.create(&value).await?),
    ))
}
#[utoipa::path(put,path="/api/recurring-transfers/{id}",params(("id"=String,Path)),request_body=RecurringTransferUpsertRequest,responses((status=200,body=RecurringTransfer),(status=400),(status=404)))]
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(value): Json<RecurringTransferUpsertRequest>,
) -> AppResult<Json<RecurringTransfer>> {
    Ok(Json(state.recurring_transfers.update(&id, &value).await?))
}
#[utoipa::path(delete,path="/api/recurring-transfers/{id}",params(("id"=String,Path)),responses((status=204),(status=404),(status=409)))]
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    state.recurring_transfers.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
