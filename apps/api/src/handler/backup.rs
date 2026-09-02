use crate::{service::backup::BackupService, utils::error::AppResult};
use axum::{extract::State, http::header, response::IntoResponse};

#[derive(Clone)]
pub struct AppState {
    pub backup: BackupService,
}

#[utoipa::path(get,path="/api/backup",responses((status=200,description="SQLiteデータベースのバックアップ",content_type="application/octet-stream")))]
pub async fn get(State(s): State<AppState>) -> AppResult<impl IntoResponse> {
    let backup = s.backup.create().await?;
    let content_length = backup.bytes.len().to_string();
    let filename = format!("kakeibo-backup-{}.db", backup.date);

    Ok((
        [
            (header::CONTENT_TYPE, "application/octet-stream".to_owned()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
            (header::CONTENT_LENGTH, content_length),
            (header::CACHE_CONTROL, "no-store".to_owned()),
        ],
        backup.bytes,
    ))
}
