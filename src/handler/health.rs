use axum::Json;

use crate::model::health::Health;

#[utoipa::path(get, path = "/health", responses((status = 200, body = Health)))]
pub async fn get() -> Json<Health> {
    Json(Health { status: "ok" })
}
