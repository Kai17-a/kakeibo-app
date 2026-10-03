use crate::{
    service::backup::BackupService,
    utils::error::{AppError, AppResult},
};
use axum::{
    Json,
    body::Body,
    extract::State,
    http::{HeaderMap, header},
    response::IntoResponse,
};
use futures_util::StreamExt;
use serde::Serialize;
use tokio::{fs::File, io::AsyncWriteExt};

pub const MAX_RESTORE_BYTES: usize = 256 * 1024 * 1024;

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

#[derive(Serialize, utoipa::ToSchema)]
pub struct RestoreResponse {
    message: &'static str,
}

#[utoipa::path(
    post,
    path = "/api/backup/restore",
    request_body(content = Vec<u8>, content_type = "application/octet-stream"),
    responses(
        (status = 200, description = "フルバックアップから復元", body = RestoreResponse),
        (status = 400, description = "バックアップ検証エラー"),
        (status = 403, description = "別サイトからのリクエスト"),
        (status = 409, description = "別の復元を実行中"),
        (status = 413, description = "アップロードサイズ超過"),
        (status = 415, description = "Content-Type が application/octet-stream ではない")
    )
)]
pub async fn restore(
    State(s): State<AppState>,
    headers: HeaderMap,
    body: Body,
) -> AppResult<Json<RestoreResponse>> {
    validate_restore_request(&headers)?;
    if headers
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok())
        .is_some_and(|size| size > MAX_RESTORE_BYTES)
    {
        return Err(too_large());
    }
    let session = s.backup.begin_restore()?;
    save_upload(body, &session.upload_path()).await?;
    // Runs detached so that a client disconnect cannot cancel the replacement half-way.
    let service = s.backup.clone();
    tokio::spawn(async move { service.restore(&session).await })
        .await
        .map_err(|error| AppError::context("Restore task failed", error))??;
    Ok(Json(RestoreResponse {
        message: "フルバックアップを復元しました。",
    }))
}

/// Refuses requests a third-party page could make a browser send: a plain form or text body,
/// or a request the browser marks as coming from another site.
fn validate_restore_request(headers: &HeaderMap) -> AppResult<()> {
    let header_text = |name: header::HeaderName| headers.get(name).and_then(|v| v.to_str().ok());
    let media_type = header_text(header::CONTENT_TYPE)
        .and_then(|value| value.split(';').next())
        .map(|value| value.trim().to_ascii_lowercase());
    if media_type.as_deref() != Some("application/octet-stream") {
        return Err(AppError::unsupported_media_type(
            "バックアップは application/octet-stream で送信してください。現在のデータは変更されていません。",
        ));
    }
    let cross_site = || {
        AppError::forbidden(
            "別のサイトからの復元リクエストは受け付けません。現在のデータは変更されていません。",
        )
    };
    if let Some(site) = headers
        .get("sec-fetch-site")
        .and_then(|value| value.to_str().ok())
    {
        return if matches!(site, "same-origin" | "none") {
            Ok(())
        } else {
            Err(cross_site())
        };
    }
    if let Some(origin) = header_text(header::ORIGIN) {
        let same_host = match (origin.split_once("://"), header_text(header::HOST)) {
            (Some((_, origin_host)), Some(host)) => origin_host.eq_ignore_ascii_case(host),
            _ => false,
        };
        if !same_host {
            return Err(cross_site());
        }
    }
    Ok(())
}

async fn save_upload(body: Body, path: &std::path::Path) -> AppResult<()> {
    let mut file = File::create(path)
        .await
        .map_err(|error| AppError::context("Failed to create upload file", error))?;
    let mut stream = body.into_data_stream();
    let mut size = 0_usize;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| {
            AppError::bad_request(&format!(
                "アップロードを読み込めませんでした: {error}。現在のデータは変更されていません。"
            ))
        })?;
        size = size.saturating_add(chunk.len());
        if size > MAX_RESTORE_BYTES {
            return Err(too_large());
        }
        file.write_all(&chunk)
            .await
            .map_err(|error| AppError::context("Failed to write upload file", error))?;
    }
    file.flush()
        .await
        .map_err(|error| AppError::context("Failed to finish upload file", error))?;
    if size == 0 {
        return Err(AppError::bad_request(
            "空のファイルは復元できません。現在のデータは変更されていません。",
        ));
    }
    Ok(())
}

fn too_large() -> AppError {
    AppError::payload_too_large(
        "バックアップは256 MiB以下のファイルを選択してください。現在のデータは変更されていません。",
    )
}
