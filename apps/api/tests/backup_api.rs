use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use kakeibo_app::{
    database::migration::connect_path,
    router::{backup, incomes},
    service::backup::{BackupService, DATA_TABLES, DELETE_ORDER, RestoreConfig},
};
use serde_json::json;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
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

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "kakeibo-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn restore_config(root: &Path) -> RestoreConfig {
    RestoreConfig {
        working_directory: root.join("work"),
        safety_backup_directory: root.join("safety"),
        safety_backup_generations: 5,
    }
}

async fn migrated_pool(path: &Path) -> SqlitePool {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    connect_path(path).await.unwrap()
}

fn app(pool: SqlitePool, root: &Path) -> Router {
    fs::create_dir_all(root.join("work")).unwrap();
    backup::create(pool.clone(), restore_config(root)).merge(incomes::create(pool))
}

async fn request(app: &Router, request: Request<Body>) -> (StatusCode, Vec<u8>) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, bytes.to_vec())
}

async fn full_backup(app: &Router) -> Vec<u8> {
    let (status, bytes) = request(
        app,
        Request::get("/api/backup").body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    bytes
}

async fn restore(app: &Router, bytes: Vec<u8>) -> (StatusCode, Vec<u8>) {
    request(
        app,
        Request::post("/api/backup/restore")
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .body(Body::from(bytes))
            .unwrap(),
    )
    .await
}

async fn seed_income(pool: &SqlitePool, suffix: &str, amount: &str) {
    sqlx::query("INSERT INTO income_categories (id, name) VALUES (?1, ?2)")
        .bind(format!("category-{suffix}"))
        .bind(format!("category {suffix}"))
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO incomes (id, category_id, transaction_date, amount) VALUES (?1, ?2, '2026-01-01', ?3)")
        .bind(format!("income-{suffix}"))
        .bind(format!("category-{suffix}"))
        .bind(amount)
        .execute(pool)
        .await
        .unwrap();
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

    let work = std::env::temp_dir().join(format!(
        "kakeibo-backup-work-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&work).unwrap();
    let response = backup::create(
        pool.clone(),
        RestoreConfig {
            working_directory: work.clone(),
            safety_backup_directory: work.join("safety"),
            safety_backup_generations: 5,
        },
    )
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
    fs::remove_dir_all(work).unwrap();
}

#[tokio::test]
async fn restores_all_data_atomically_keeps_pool_usable_and_creates_safety_backup() {
    let root = TestDirectory::new("restore-success");
    let pool = migrated_pool(&root.0.join("live.db")).await;
    seed_income(&pool, "backup", "100").await;
    let app = app(pool.clone(), &root.0);
    let backup_bytes = full_backup(&app).await;

    seed_income(&pool, "current", "200").await;
    let (status, body) = restore(&app, backup_bytes).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let ids: Vec<String> = sqlx::query_scalar("SELECT id FROM incomes ORDER BY id")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(ids, ["income-backup"]);

    let (status, body) = request(
        &app,
        Request::get("/api/incomes").body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let create = json!({
        "category_id": "category-backup", "transaction_date": "2026-02-01",
        "amount": "300", "payment_method_id": null, "recurring_income_id": null,
        "description": "after restore"
    });
    let (status, body) = request(
        &app,
        Request::post("/api/incomes")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(create.to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "{}",
        String::from_utf8_lossy(&body)
    );

    let safety_files = fs::read_dir(root.0.join("safety"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(safety_files.len(), 1);
    let safety = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&safety_files[0])
                .read_only(true),
        )
        .await
        .unwrap();
    let safety_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM incomes ORDER BY id")
        .fetch_all(&safety)
        .await
        .unwrap();
    assert_eq!(safety_ids, ["income-backup", "income-current"]);
    safety.close().await;
    assert_eq!(fs::read_dir(root.0.join("work")).unwrap().count(), 0);
    pool.close().await;
}

#[tokio::test]
async fn rejects_invalid_files_without_changing_current_data() {
    let root = TestDirectory::new("restore-invalid");
    let pool = migrated_pool(&root.0.join("live.db")).await;
    seed_income(&pool, "current", "200").await;
    let app = app(pool.clone(), &root.0);

    for bytes in [
        Vec::new(),
        b"not sqlite".to_vec(),
        b"SQLite format 3\0broken".to_vec(),
    ] {
        let (status, _) = restore(&app, bytes).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM incomes")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    let other_path = root.0.join("other.db");
    let other = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&other_path)
                .create_if_missing(true),
        )
        .await
        .unwrap();
    sqlx::query("CREATE TABLE alien (id INTEGER)")
        .execute(&other)
        .await
        .unwrap();
    other.close().await;
    let (status, _) = restore(&app, fs::read(other_path).unwrap()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM incomes")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    assert_eq!(fs::read_dir(root.0.join("work")).unwrap().count(), 0);
    pool.close().await;
}

#[tokio::test]
async fn rejects_tampered_migrations_unknown_versions_foreign_keys_and_active_schema_objects() {
    let root = TestDirectory::new("restore-validation");
    let pool = migrated_pool(&root.0.join("live.db")).await;
    seed_income(&pool, "current", "200").await;
    let app = app(pool.clone(), &root.0);
    let valid = full_backup(&app).await;

    for (name, statement) in [
        (
            "checksum",
            "UPDATE _sqlx_migrations SET checksum = x'00' WHERE version = 1",
        ),
        (
            "unknown",
            "INSERT INTO _sqlx_migrations(version, description, installed_on, success, checksum, execution_time) VALUES (9999, 'unknown', CURRENT_TIMESTAMP, 1, x'00', 0)",
        ),
        (
            "trigger",
            "CREATE TRIGGER malicious AFTER INSERT ON incomes BEGIN DELETE FROM incomes; END",
        ),
        ("view", "CREATE VIEW leaked AS SELECT * FROM incomes"),
        ("table", "CREATE TABLE alien (id INTEGER)"),
    ] {
        let path = root.0.join(format!("{name}.db"));
        fs::write(&path, &valid).unwrap();
        let candidate = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(SqliteConnectOptions::new().filename(&path))
            .await
            .unwrap();
        sqlx::query(sqlx::AssertSqlSafe(statement.to_owned()))
            .execute(&candidate)
            .await
            .unwrap();
        candidate.close().await;
        let (status, _) = restore(&app, fs::read(path).unwrap()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{name}");
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM incomes")
                .fetch_one(&pool)
                .await
                .unwrap(),
            1
        );
    }

    let path = root.0.join("foreign-key.db");
    fs::write(&path, &valid).unwrap();
    let candidate = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&path)
                .foreign_keys(false),
        )
        .await
        .unwrap();
    sqlx::query("INSERT INTO incomes (id, category_id, transaction_date, amount) VALUES ('invalid', 'missing', '2026-01-01', '1')")
        .execute(&candidate).await.unwrap();
    candidate.close().await;
    let (status, _) = restore(&app, fs::read(path).unwrap()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM incomes")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    pool.close().await;
}

#[tokio::test]
async fn upgrades_version_twelve_backup_before_restoring() {
    let root = TestDirectory::new("restore-old");
    let pool = migrated_pool(&root.0.join("live.db")).await;
    seed_income(&pool, "old", "100").await;
    sqlx::query("INSERT INTO payment_methods (id, name) VALUES ('bank-old', '旧銀行'), ('card-old', '旧カード')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DROP INDEX idx_transfers_transaction_date; DROP INDEX idx_transfers_to_payment_method_id; DROP INDEX idx_transfers_from_payment_method_id; DROP TABLE transfers; ALTER TABLE payment_methods DROP COLUMN is_investment; DELETE FROM _sqlx_migrations WHERE version = 13")
        .execute(&pool).await.unwrap();
    pool.close().await;
    let old_bytes = fs::read(root.0.join("live.db")).unwrap();

    let current = migrated_pool(&root.0.join("current.db")).await;
    seed_income(&current, "current", "200").await;
    let app = app(current.clone(), &root.0);
    let (status, body) = restore(&app, old_bytes).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM incomes WHERE id = 'income-old'")
            .fetch_one(&current)
            .await
            .unwrap(),
        1
    );
    // Columns and tables added by migration 13 must exist for the rows of the older backup.
    let methods: Vec<(String, i64)> =
        sqlx::query_as("SELECT id, is_investment FROM payment_methods ORDER BY id")
            .fetch_all(&current)
            .await
            .unwrap();
    assert_eq!(
        methods,
        [("bank-old".to_owned(), 0), ("card-old".to_owned(), 0)]
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM incomes WHERE id = 'income-current'")
            .fetch_one(&current)
            .await
            .unwrap(),
        0
    );
    sqlx::query("INSERT INTO transfers (transaction_date, amount, from_payment_method_id, to_payment_method_id) VALUES ('2026-01-01', '100', 'bank-old', 'card-old')")
        .execute(&current)
        .await
        .unwrap();
    current.close().await;
}

#[tokio::test]
async fn rolls_back_live_changes_when_replacement_fails() {
    let root = TestDirectory::new("restore-rollback");
    let pool = migrated_pool(&root.0.join("live.db")).await;
    seed_income(&pool, "backup", "100").await;
    let app = app(pool.clone(), &root.0);
    let backup_bytes = full_backup(&app).await;
    seed_income(&pool, "current", "200").await;
    sqlx::query(
        "CREATE TRIGGER fail_restore BEFORE DELETE ON incomes BEGIN SELECT RAISE(ABORT, 'forced restore failure'); END",
    )
        .execute(&pool)
        .await
        .unwrap();

    let (status, _) = restore(&app, backup_bytes).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let ids: Vec<String> = sqlx::query_scalar("SELECT id FROM incomes ORDER BY id")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(ids, ["income-backup", "income-current"]);
    pool.close().await;
}

#[tokio::test]
async fn rejects_declared_upload_larger_than_limit() {
    let root = TestDirectory::new("restore-size");
    let pool = migrated_pool(&root.0.join("live.db")).await;
    let app = app(pool.clone(), &root.0);
    let (status, body) = request(
        &app,
        Request::post("/api/backup/restore")
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .header(
                header::CONTENT_LENGTH,
                (256_usize * 1024 * 1024 + 1).to_string(),
            )
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::PAYLOAD_TOO_LARGE,
        "{}",
        String::from_utf8_lossy(&body)
    );
    pool.close().await;
}

async fn income_ids(pool: &SqlitePool) -> Vec<String> {
    sqlx::query_scalar("SELECT id FROM incomes ORDER BY id")
        .fetch_all(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn rejects_requests_a_third_party_page_could_send() {
    let root = TestDirectory::new("restore-csrf");
    let pool = migrated_pool(&root.0.join("live.db")).await;
    seed_income(&pool, "backup", "100").await;
    let app = app(pool.clone(), &root.0);
    let backup_bytes = full_backup(&app).await;
    seed_income(&pool, "current", "200").await;

    let post = |headers: &[(&str, &str)]| {
        let mut builder = Request::post("/api/backup/restore");
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        builder.body(Body::from(backup_bytes.clone())).unwrap()
    };
    let octet = ("content-type", "application/octet-stream");
    for (name, headers, expected) in [
        (
            "no content type",
            vec![],
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (
            "text/plain",
            vec![("content-type", "text/plain")],
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (
            "form",
            vec![("content-type", "multipart/form-data; boundary=x")],
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (
            "cross-site fetch metadata",
            vec![octet, ("sec-fetch-site", "cross-site")],
            StatusCode::FORBIDDEN,
        ),
        (
            "same-site fetch metadata",
            vec![octet, ("sec-fetch-site", "same-site")],
            StatusCode::FORBIDDEN,
        ),
        (
            "foreign origin",
            vec![
                octet,
                ("host", "kakeibo.local:8000"),
                ("origin", "http://evil.example"),
            ],
            StatusCode::FORBIDDEN,
        ),
        (
            "opaque origin",
            vec![octet, ("host", "kakeibo.local:8000"), ("origin", "null")],
            StatusCode::FORBIDDEN,
        ),
    ] {
        let (status, body) = request(&app, post(&headers)).await;
        assert_eq!(
            status,
            expected,
            "{name}: {}",
            String::from_utf8_lossy(&body)
        );
        assert_eq!(
            income_ids(&pool).await,
            ["income-backup", "income-current"],
            "{name}"
        );
    }
    assert!(!root.0.join("safety").exists());

    for (name, headers) in [
        (
            "same origin",
            vec![
                ("content-type", "application/octet-stream; charset=binary"),
                ("host", "kakeibo.local:8000"),
                ("origin", "http://KAKEIBO.local:8000"),
            ],
        ),
        (
            "same-origin fetch metadata behind the dev proxy",
            vec![
                octet,
                ("sec-fetch-site", "same-origin"),
                ("host", "localhost:8000"),
                ("origin", "http://localhost:3000"),
            ],
        ),
        ("no origin", vec![octet]),
    ] {
        seed_income(&pool, name, "1").await;
        let (status, body) = request(&app, post(&headers)).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "{name}: {}",
            String::from_utf8_lossy(&body)
        );
        assert_eq!(income_ids(&pool).await, ["income-backup"], "{name}");
    }
    pool.close().await;
}

#[tokio::test]
async fn allows_one_restore_at_a_time_and_cleans_its_working_directory() {
    let root = TestDirectory::new("restore-slot");
    let pool = migrated_pool(&root.0.join("live.db")).await;
    seed_income(&pool, "current", "200").await;
    let work = root.0.join("work");
    fs::create_dir_all(&work).unwrap();
    let service = BackupService::new(pool.clone(), restore_config(&root.0));
    let app = app(pool.clone(), &root.0);
    let backup_bytes = full_backup(&app).await;

    let session = service.begin_restore().unwrap();
    assert!(session.upload_path().starts_with(&work));
    assert_eq!(fs::read_dir(&work).unwrap().count(), 1);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let directory = session.upload_path().parent().unwrap().to_owned();
        assert_eq!(
            fs::metadata(directory).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
    // Clones share the slot, as the router state is cloned per request.
    assert!(service.clone().begin_restore().is_err());
    fs::write(session.upload_path(), b"left behind by a cancelled upload").unwrap();
    drop(session);
    assert_eq!(fs::read_dir(&work).unwrap().count(), 0);

    let held = service.begin_restore().unwrap();
    drop(held);
    let (status, body) = restore(&app, backup_bytes).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    assert_eq!(fs::read_dir(&work).unwrap().count(), 0);
    pool.close().await;
}

#[tokio::test]
async fn refuses_a_second_restore_while_one_is_running() {
    let root = TestDirectory::new("restore-busy");
    let pool = migrated_pool(&root.0.join("live.db")).await;
    seed_income(&pool, "current", "200").await;
    fs::create_dir_all(root.0.join("work")).unwrap();
    let service = BackupService::new(pool.clone(), restore_config(&root.0));
    let app = backup::create_with_service(service.clone());
    let backup_bytes = full_backup(&app).await;

    let running = service.begin_restore().unwrap();
    let (status, body) = restore(&app, backup_bytes.clone()).await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "{}",
        String::from_utf8_lossy(&body)
    );
    assert_eq!(income_ids(&pool).await, ["income-current"]);
    drop(running);

    let (status, body) = restore(&app, backup_bytes).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    pool.close().await;
}

#[cfg(unix)]
#[tokio::test]
async fn safety_backup_is_private_to_the_server_user() {
    use std::os::unix::fs::PermissionsExt;
    let root = TestDirectory::new("restore-permissions");
    let pool = migrated_pool(&root.0.join("live.db")).await;
    let app = app(pool.clone(), &root.0);
    let backup_bytes = full_backup(&app).await;
    let (status, body) = restore(&app, backup_bytes).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));

    let safety = root.0.join("safety");
    assert_eq!(
        fs::metadata(&safety).unwrap().permissions().mode() & 0o777,
        0o700
    );
    for entry in fs::read_dir(&safety).unwrap() {
        let mode = entry.unwrap().metadata().unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
    pool.close().await;
}

fn listed_columns(columns: &str) -> Vec<String> {
    let mut columns = columns
        .split(',')
        .map(|column| column.trim().to_owned())
        .collect::<Vec<_>>();
    columns.sort();
    columns
}

/// Fails when a migration adds a table or column that the restore would silently drop.
#[tokio::test]
async fn restore_covers_every_table_and_column_of_the_current_schema() {
    let root = TestDirectory::new("restore-schema");
    let pool = migrated_pool(&root.0.join("live.db")).await;

    let mut tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_schema WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name <> '_sqlx_migrations'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    tables.sort();
    let mut copied = DATA_TABLES
        .iter()
        .map(|(table, _)| (*table).to_owned())
        .collect::<Vec<_>>();
    copied.sort();
    let mut deleted = DELETE_ORDER
        .iter()
        .map(|table| (*table).to_owned())
        .collect::<Vec<_>>();
    deleted.sort();
    assert_eq!(copied, tables, "DATA_TABLES must list every table");
    assert_eq!(deleted, tables, "DELETE_ORDER must list every table");

    for (table, columns) in DATA_TABLES {
        let mut actual: Vec<String> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT name FROM pragma_table_xinfo('{table}') WHERE hidden = 0"
        )))
        .fetch_all(&pool)
        .await
        .unwrap();
        actual.sort();
        assert_eq!(listed_columns(columns), actual, "columns of {table}");
    }
    pool.close().await;
}

async fn dump(pool: &SqlitePool) -> Vec<(String, Vec<String>)> {
    let mut tables = Vec::new();
    for (table, columns) in DATA_TABLES {
        let row = columns
            .split(',')
            .map(|column| format!("quote({})", column.trim()))
            .collect::<Vec<_>>()
            .join(" || '|' || ");
        let rows: Vec<String> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT {row} FROM {table} ORDER BY 1"
        )))
        .fetch_all(pool)
        .await
        .unwrap();
        tables.push(((*table).to_owned(), rows));
    }
    tables
}

#[tokio::test]
async fn restores_every_column_of_every_table() {
    let root = TestDirectory::new("restore-roundtrip");
    let pool = migrated_pool(&root.0.join("live.db")).await;
    sqlx::query(
        "INSERT INTO expense_categories (id, created_at, updated_at, name, description, parent_category_id, display_order) VALUES
           ('ec-parent', '2025-01-01 00:00:00', '2025-01-02 00:00:00', '食費', '親', NULL, 3),
           ('ec-child', '2025-01-03 00:00:00', '2025-01-04 00:00:00', '外食', '子', 'ec-parent', 7);
         INSERT INTO income_categories (id, created_at, updated_at, name, description, parent_category_id, display_order) VALUES
           ('ic-parent', '2025-02-01 00:00:00', '2025-02-02 00:00:00', '給与', '親', NULL, 2),
           ('ic-child', '2025-02-03 00:00:00', '2025-02-04 00:00:00', '賞与', '子', 'ic-parent', 5);
         INSERT INTO payment_methods (id, created_at, updated_at, name, description, initial_balance, is_investment) VALUES
           ('pm-bank', '2025-03-01 00:00:00', '2025-03-02 00:00:00', '銀行', '普通預金', '100000', 0),
           ('pm-nisa', '2025-03-03 00:00:00', '2025-03-04 00:00:00', 'NISA', '投資', '0', 1);
         INSERT INTO recurring_expenses (id, created_at, updated_at, name, amount, payment_day, start_date, end_date, category_id, payment_method_id, is_active, description, is_variable, foreign_amount, currency_code, exchange_rate) VALUES
           ('re-1', '2025-04-01 00:00:00', '2025-04-02 00:00:00', '動画', '1500', 31, '2025-01-01', '2026-12-31', 'ec-child', 'pm-bank', 0, '備考', 1, '10', 'USD', '150.5');
         INSERT INTO recurring_incomes (id, created_at, updated_at, name, amount, payment_day, start_date, end_date, category_id, is_active, is_variable, description) VALUES
           ('ri-1', '2025-05-01 00:00:00', '2025-05-02 00:00:00', '副業', '30000', 25, '2025-01-01', '2026-12-31', 'ic-child', 0, 1, '備考');
         INSERT INTO incomes (id, created_at, updated_at, category_id, transaction_date, amount, description, recurring_income_id, payment_method_id) VALUES
           ('in-1', '2025-06-01 00:00:00', '2025-06-02 00:00:00', 'ic-child', '2025-06-25', '30000', '6月分', 'ri-1', 'pm-bank');
         INSERT INTO expenses (id, created_at, updated_at, transaction_date, amount, category_id, payment_method_id, recurring_expense_id, description, foreign_amount, currency_code, exchange_rate, exchange_rate_date) VALUES
           ('ex-1', '2025-07-01 00:00:00', '2025-07-02 00:00:00', '2025-07-31', '1505', 'ec-child', 'pm-bank', 're-1', '7月分', '10', 'USD', '150.5', '2025-07-30');
         INSERT INTO budgets (id, created_at, updated_at, category_id, amount) VALUES
           ('bg-1', '2025-08-01 00:00:00', '2025-08-02 00:00:00', 'ec-parent', '40000');
         INSERT INTO exchange_rates (target_date, base_currency, quote_currency, rate, effective_date, source, fetched_at) VALUES
           ('2025-07-31', 'USD', 'JPY', '150.5', '2025-07-30', 'test', '2025-07-31 01:02:03');
         INSERT INTO transfers (id, created_at, updated_at, transaction_date, amount, from_payment_method_id, to_payment_method_id, description) VALUES
           ('tr-1', '2025-09-01 00:00:00', '2025-09-02 00:00:00', '2025-09-15', '33333', 'pm-bank', 'pm-nisa', '積立');",
    )
    .execute(&pool)
    .await
    .unwrap();
    let expected = dump(&pool).await;
    for (table, rows) in &expected {
        assert!(!rows.is_empty(), "{table} needs a row in this test");
        assert!(
            !rows.iter().any(|row| row.contains("NULL")) || table.ends_with("_categories"),
            "{table} needs a non-null value in every column: {rows:?}"
        );
    }
    let app = app(pool.clone(), &root.0);
    let backup_bytes = full_backup(&app).await;

    sqlx::query(
        "DELETE FROM transfers; DELETE FROM budgets; DELETE FROM expenses; DELETE FROM incomes;
         DELETE FROM exchange_rates;
         UPDATE payment_methods SET name = 'changed', is_investment = 1 - is_investment;
         UPDATE recurring_expenses SET amount = '1', is_active = 1, is_variable = 0;
         INSERT INTO payment_methods (id, name) VALUES ('pm-extra', '追加');",
    )
    .execute(&pool)
    .await
    .unwrap();
    assert_ne!(dump(&pool).await, expected);

    let (status, body) = restore(&app, backup_bytes).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    assert_eq!(dump(&pool).await, expected);
    pool.close().await;
}
