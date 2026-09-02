use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::router::backup;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::{collections::HashSet, fs, path::PathBuf};
use tower::ServiceExt;

fn temporary_backups() -> HashSet<PathBuf> {
    let prefix = format!("kakeibo-backup-{}-", std::process::id());
    fs::read_dir(std::env::temp_dir())
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&prefix) && name.ends_with(".db"))
        })
        .collect()
}

#[tokio::test]
async fn downloads_sqlite_backup_and_removes_temporary_file() {
    let database_path = std::env::temp_dir().join(format!(
        "kakeibo-backup-test-{}-{}.db",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&database_path)
                .create_if_missing(true),
        )
        .await
        .unwrap();
    sqlx::query("CREATE TABLE entries(id INTEGER PRIMARY KEY, value TEXT NOT NULL)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO entries(value) VALUES ('backup test')")
        .execute(&pool)
        .await
        .unwrap();
    let expected_date =
        sqlx::query_scalar::<_, String>("SELECT strftime('%Y%m%d', 'now', 'localtime')")
            .fetch_one(&pool)
            .await
            .unwrap();
    let before = temporary_backups();

    let response = backup::create(pool.clone())
        .oneshot(Request::get("/api/backup").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    assert_eq!(headers["content-type"], "application/octet-stream");
    assert_eq!(headers["cache-control"], "no-store");
    assert_eq!(
        headers["content-disposition"],
        format!("attachment; filename=\"kakeibo-backup-{expected_date}.db\"")
    );
    let content_length = headers["content-length"]
        .to_str()
        .unwrap()
        .parse::<usize>()
        .unwrap();
    assert_eq!(content_length, body.len());
    assert!(body.len() > 16);
    assert_eq!(&body[..16], b"SQLite format 3\0");
    assert_eq!(temporary_backups(), before);
    pool.close().await;
    fs::remove_file(database_path).unwrap();
}
