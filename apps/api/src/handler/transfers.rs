use crate::{
    model::transfers::{Transfer, TransferUpsertRequest},
    service::transfers::TransferService,
    utils::error::AppResult,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
#[derive(Clone)]
pub struct AppState {
    pub transfers: TransferService,
}
#[utoipa::path(get,path="/api/transfers",responses((status=200,body=Vec<Transfer>)))]
pub async fn list(State(s): State<AppState>) -> AppResult<Json<Vec<Transfer>>> {
    Ok(Json(s.transfers.list().await?))
}
#[utoipa::path(get,path="/api/transfers/{id}",params(("id"=String,Path)),responses((status=200,body=Transfer),(status=404)))]
pub async fn get(State(s): State<AppState>, Path(id): Path<String>) -> AppResult<Json<Transfer>> {
    Ok(Json(s.transfers.get(&id).await?))
}
#[utoipa::path(post,path="/api/transfers",request_body=TransferUpsertRequest,responses((status=201,body=Transfer),(status=400)))]
pub async fn create(
    State(s): State<AppState>,
    Json(v): Json<TransferUpsertRequest>,
) -> AppResult<(StatusCode, Json<Transfer>)> {
    Ok((StatusCode::CREATED, Json(s.transfers.create(&v).await?)))
}
#[utoipa::path(put,path="/api/transfers/{id}",params(("id"=String,Path)),request_body=TransferUpsertRequest,responses((status=200,body=Transfer),(status=400),(status=404)))]
pub async fn update(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(v): Json<TransferUpsertRequest>,
) -> AppResult<Json<Transfer>> {
    Ok(Json(s.transfers.update(&id, &v).await?))
}
#[utoipa::path(delete,path="/api/transfers/{id}",params(("id"=String,Path)),responses((status=204),(status=404)))]
pub async fn delete(State(s): State<AppState>, Path(id): Path<String>) -> AppResult<StatusCode> {
    s.transfers.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
