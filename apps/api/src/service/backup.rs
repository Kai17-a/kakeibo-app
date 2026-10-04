use crate::{
    database::migration::MIGRATOR,
    utils::error::{AppError, AppResult},
};
use sqlx::{
    AssertSqlSafe, Connection, Row, SqliteConnection, SqlitePool,
    migrate::MigrateDatabase,
    sqlite::{Sqlite, SqliteConnectOptions, SqlitePoolOptions},
};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::warn;

/// Tables an uploaded backup may contain. The webhook tables were dropped by migration 12 and
/// stay listed so that backups taken before it are still accepted; they are never copied.
const KNOWN_TABLES: &[&str] = &[
    "_sqlx_migrations",
    "expense_categories",
    "payment_methods",
    "recurring_expenses",
    "expenses",
    "incomes",
    "income_categories",
    "recurring_incomes",
    "recurring_transfers",
    "budgets",
    "exchange_rates",
    "transfers",
    "webhook_urls",
    "webhook_url_events",
];
/// Every data table with all of its columns, in an order that satisfies foreign keys on insert.
pub const DATA_TABLES: &[(&str, &str)] = &[
    (
        "expense_categories",
        "id, created_at, updated_at, name, description, parent_category_id, display_order",
    ),
    (
        "income_categories",
        "id, created_at, updated_at, name, description, parent_category_id, display_order",
    ),
    (
        "payment_methods",
        "id, created_at, updated_at, name, description, initial_balance, is_investment",
    ),
    (
        "recurring_expenses",
        "id, created_at, updated_at, name, amount, payment_day, start_date, end_date, category_id, payment_method_id, is_active, description, is_variable, foreign_amount, currency_code, exchange_rate",
    ),
    (
        "recurring_incomes",
        "id, created_at, updated_at, name, amount, payment_day, start_date, end_date, category_id, is_active, is_variable, description",
    ),
    (
        "recurring_transfers",
        "id, created_at, updated_at, name, amount, payment_day, start_date, end_date, from_payment_method_id, to_payment_method_id, is_active, description",
    ),
    (
        "incomes",
        "id, created_at, updated_at, category_id, transaction_date, amount, description, recurring_income_id, payment_method_id",
    ),
    (
        "expenses",
        "id, created_at, updated_at, transaction_date, amount, category_id, payment_method_id, recurring_expense_id, description, foreign_amount, currency_code, exchange_rate, exchange_rate_date",
    ),
    ("budgets", "id, created_at, updated_at, category_id, amount"),
    (
        "exchange_rates",
        "target_date, base_currency, quote_currency, rate, effective_date, source, fetched_at",
    ),
    (
        "transfers",
        "id, created_at, updated_at, transaction_date, amount, from_payment_method_id, to_payment_method_id, description, recurring_transfer_id",
    ),
];
pub const DELETE_ORDER: &[&str] = &[
    "transfers",
    "budgets",
    "expenses",
    "incomes",
    "recurring_expenses",
    "recurring_incomes",
    "recurring_transfers",
    "exchange_rates",
    "payment_methods",
    "expense_categories",
    "income_categories",
];

pub struct Backup {
    pub bytes: Vec<u8>,
    pub date: String,
}

#[derive(Clone)]
pub struct RestoreConfig {
    pub working_directory: PathBuf,
    pub safety_backup_directory: PathBuf,
    pub safety_backup_generations: usize,
}

impl RestoreConfig {
    pub fn production() -> Self {
        Self {
            working_directory: std::env::temp_dir(),
            safety_backup_directory: PathBuf::from(".restore-backups"),
            safety_backup_generations: 5,
        }
    }
}

/// One restore in flight: holds the process-wide restore slot and a private working directory.
/// Dropping it frees the slot and deletes the directory with every file SQLite left in it,
/// also when the request is cancelled half-way.
pub struct RestoreSession {
    directory: PathBuf,
    in_progress: Arc<AtomicBool>,
}

impl RestoreSession {
    pub fn upload_path(&self) -> PathBuf {
        self.directory.join("upload.db")
    }

    fn migrated_path(&self) -> PathBuf {
        self.directory.join("migrated.db")
    }

    fn sanitized_path(&self) -> PathBuf {
        self.directory.join("sanitized.db")
    }
}

impl Drop for RestoreSession {
    fn drop(&mut self) {
        match fs::remove_dir_all(&self.directory) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => warn!(%error, "Failed to remove restore working directory"),
        }
        self.in_progress.store(false, Ordering::Release);
    }
}

#[derive(Clone)]
pub struct BackupService {
    pool: SqlitePool,
    restore_config: RestoreConfig,
    restore_in_progress: Arc<AtomicBool>,
}

impl BackupService {
    pub fn new(pool: SqlitePool, restore_config: RestoreConfig) -> Self {
        Self {
            pool,
            restore_config,
            restore_in_progress: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Claims the single restore slot; a second restore is refused instead of queued.
    pub fn begin_restore(&self) -> AppResult<RestoreSession> {
        if self
            .restore_in_progress
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(AppError::conflict(
                "別の復元を実行中です。完了してからやり直してください。現在のデータは変更されていません。",
            ));
        }
        let session = RestoreSession {
            directory: self.restore_config.working_directory.join(format!(
                "kakeibo-restore-{}-{}",
                std::process::id(),
                unique_suffix()?
            )),
            in_progress: self.restore_in_progress.clone(),
        };
        create_private_directory(&session.directory)
            .map_err(|error| AppError::context("Failed to create restore directory", error))?;
        Ok(session)
    }

    pub async fn create(&self) -> AppResult<Backup> {
        let date = sqlx::query_scalar::<_, String>("SELECT strftime('%Y%m%d', 'now', 'localtime')")
            .fetch_one(&self.pool)
            .await?;
        let path = temporary_path(&self.restore_config.working_directory, "download")?;
        let result = self.create_file(&path).await;
        let cleanup_result = remove_file(&path);

        match (result, cleanup_result) {
            (Ok(bytes), Ok(())) => Ok(Backup { bytes, date }),
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(error),
        }
    }

    async fn create_file(&self, path: &Path) -> AppResult<Vec<u8>> {
        vacuum_into(&self.pool, path, "Failed to create database backup").await?;
        fs::read(path).map_err(|error| AppError::context("Failed to read database backup", error))
    }

    pub async fn restore(&self, session: &RestoreSession) -> AppResult<()> {
        self.prepare_and_restore(
            &session.upload_path(),
            &session.migrated_path(),
            &session.sanitized_path(),
        )
        .await
    }

    async fn prepare_and_restore(
        &self,
        uploaded_path: &Path,
        migrated_path: &Path,
        sanitized_path: &Path,
    ) -> AppResult<()> {
        validate_uploaded(uploaded_path).await?;
        fs::copy(uploaded_path, migrated_path)
            .map_err(|error| AppError::context("Failed to prepare uploaded backup", error))?;
        migrate_copy(migrated_path).await?;
        create_sanitized_copy(migrated_path, sanitized_path).await?;
        validate_current_copy(sanitized_path).await?;
        self.replace_live_data(sanitized_path).await
    }

    async fn replace_live_data(&self, sanitized_path: &Path) -> AppResult<()> {
        fs::create_dir_all(&self.restore_config.safety_backup_directory).map_err(|error| {
            AppError::context("Failed to create safety backup directory", error)
        })?;
        restrict_permissions(&self.restore_config.safety_backup_directory, 0o700)?;
        let backup_name = format!("kakeibo-before-restore-{}.db", unique_suffix()?);
        let backup_path = self
            .restore_config
            .safety_backup_directory
            .join(backup_name);
        let mut connection = self.pool.acquire().await?;
        // A restore cancelled mid-way can leave its database attached to the pooled connection.
        let _ = sqlx::query("DETACH DATABASE restored")
            .execute(&mut *connection)
            .await;
        vacuum_into(
            &mut *connection,
            &backup_path,
            "Failed to create pre-restore safety backup",
        )
        .await?;
        restrict_permissions(&backup_path, 0o600)?;
        prune_safety_backups(&self.restore_config)?;

        attach_database(&mut connection, sanitized_path, "restored").await?;
        let restore_result = async {
            let mut transaction = connection.begin().await?;
            sqlx::query("PRAGMA defer_foreign_keys = ON")
                .execute(&mut *transaction)
                .await?;
            for table in DELETE_ORDER {
                sqlx::query(AssertSqlSafe(format!("DELETE FROM {table}")))
                    .execute(&mut *transaction)
                    .await?;
            }
            for (table, columns) in DATA_TABLES {
                sqlx::query(AssertSqlSafe(format!(
                    "INSERT INTO {table} ({columns}) SELECT {columns} FROM restored.{table}"
                )))
                .execute(&mut *transaction)
                .await?;
            }
            let violations: i64 =
                sqlx::query_scalar("SELECT count(*) FROM pragma_foreign_key_check")
                    .fetch_one(&mut *transaction)
                    .await?;
            if violations != 0 {
                transaction.rollback().await?;
                return Err(AppError::bad_request(
                    "復元データに外部キー違反があるため、現在のデータは変更されませんでした。",
                ));
            }
            transaction.commit().await?;
            Ok(())
        }
        .await;
        let detach_result = sqlx::query("DETACH DATABASE restored")
            .execute(&mut *connection)
            .await;
        if let Err(error) = detach_result {
            // The connection still references the restore file, so it must not be reused.
            warn!(%error, "Failed to detach restored database; closing the connection");
            if let Err(error) = connection.close().await {
                warn!(%error, "Failed to close the connection after a failed detach");
            }
        }
        restore_result
    }
}

async fn validate_uploaded(path: &Path) -> AppResult<()> {
    let pool = open_pool(path, true).await.map_err(invalid_database)?;
    validate_integrity(&pool).await?;
    validate_schema_objects(&pool).await?;
    validate_migrations(&pool).await?;
    pool.close().await;
    Ok(())
}

async fn validate_current_copy(path: &Path) -> AppResult<()> {
    let pool = open_pool(path, true).await.map_err(invalid_database)?;
    validate_integrity(&pool).await?;
    validate_schema_objects(&pool).await?;
    validate_migrations(&pool).await?;
    pool.close().await;
    Ok(())
}

async fn validate_integrity(pool: &SqlitePool) -> AppResult<()> {
    let result: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(pool)
        .await
        .map_err(invalid_database)?;
    if result != "ok" {
        return Err(AppError::bad_request(
            "SQLiteデータベースの整合性検査に失敗したため、現在のデータは変更されませんでした。",
        ));
    }
    let violations: i64 = sqlx::query_scalar("SELECT count(*) FROM pragma_foreign_key_check")
        .fetch_one(pool)
        .await
        .map_err(invalid_database)?;
    if violations != 0 {
        return Err(AppError::bad_request(
            "バックアップに外部キー違反があるため、現在のデータは変更されませんでした。",
        ));
    }
    Ok(())
}

async fn validate_schema_objects(pool: &SqlitePool) -> AppResult<()> {
    let rows =
        sqlx::query("SELECT type, name, sql FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%'")
            .fetch_all(pool)
            .await
            .map_err(invalid_database)?;
    let known: HashSet<&str> = KNOWN_TABLES.iter().copied().collect();
    for row in rows {
        let object_type: String = row.try_get("type").map_err(invalid_database)?;
        let name: String = row.try_get("name").map_err(invalid_database)?;
        let sql: Option<String> = row.try_get("sql").map_err(invalid_database)?;
        if object_type == "trigger" || object_type == "view" {
            return Err(AppError::bad_request(
                "トリガーまたはビューを含むバックアップは安全のため復元できません。現在のデータは変更されませんでした。",
            ));
        }
        if object_type == "table"
            && (!known.contains(name.as_str())
                || sql
                    .as_deref()
                    .is_some_and(|value| value.to_ascii_uppercase().contains("VIRTUAL TABLE")))
        {
            return Err(AppError::bad_request(
                "想定外のテーブルを含むバックアップは復元できません。現在のデータは変更されませんでした。",
            ));
        }
    }
    Ok(())
}

async fn validate_migrations(pool: &SqlitePool) -> AppResult<()> {
    let exists: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sqlite_schema WHERE type = 'table' AND name = '_sqlx_migrations'",
    )
    .fetch_one(pool)
    .await
    .map_err(invalid_database)?;
    if exists != 1 {
        return Err(AppError::bad_request(
            "このアプリのマイグレーション履歴がないSQLiteファイルは復元できません。",
        ));
    }
    let rows =
        sqlx::query("SELECT version, success, checksum FROM _sqlx_migrations ORDER BY version")
            .fetch_all(pool)
            .await
            .map_err(invalid_database)?;
    let known = MIGRATOR.iter().collect::<Vec<_>>();
    if rows.is_empty() {
        return Err(AppError::bad_request(
            "マイグレーション履歴が空のバックアップは復元できません。",
        ));
    }
    for row in &rows {
        let version: i64 = row.try_get("version").map_err(invalid_database)?;
        let success: bool = row.try_get("success").map_err(invalid_database)?;
        let checksum: Vec<u8> = row.try_get("checksum").map_err(invalid_database)?;
        let Some(migration) = known.iter().find(|migration| migration.version == version) else {
            return Err(AppError::bad_request(
                "アプリより新しい未知のマイグレーションを含むバックアップは復元できません。",
            ));
        };
        if !success || migration.checksum.as_ref() != checksum.as_slice() {
            return Err(AppError::bad_request(
                "マイグレーション履歴またはチェックサムがこのアプリと一致しないため復元できません。",
            ));
        }
    }
    Ok(())
}

async fn migrate_copy(path: &Path) -> AppResult<()> {
    let pool = open_pool(path, false).await.map_err(invalid_database)?;
    MIGRATOR.run(&pool).await.map_err(|_| {
        AppError::bad_request(
            "バックアップを現在のスキーマへ更新できないため、現在のデータは変更されませんでした。",
        )
    })?;
    pool.close().await;
    Ok(())
}

async fn create_sanitized_copy(source: &Path, destination: &Path) -> AppResult<()> {
    let pool = open_pool_create(destination).await?;
    MIGRATOR
        .run(&pool)
        .await
        .map_err(|error| AppError::context("Failed to initialize restore database", error))?;
    let mut connection = pool.acquire().await?;
    attach_database(&mut connection, source, "uploaded").await?;
    let copy_result = async {
        let mut transaction = connection.begin().await?;
        sqlx::query("PRAGMA defer_foreign_keys = ON")
            .execute(&mut *transaction)
            .await?;
        for (table, columns) in DATA_TABLES {
            sqlx::query(AssertSqlSafe(format!(
                "INSERT INTO {table} ({columns}) SELECT {columns} FROM uploaded.{table}"
            )))
            .execute(&mut *transaction)
            .await
            .map_err(|_| {
                AppError::bad_request(
                    "バックアップのテーブル構造またはデータがこのアプリと一致しません。現在のデータは変更されませんでした。",
                )
            })?;
        }
        transaction.commit().await.map_err(|_| {
            AppError::bad_request(
                "バックアップのデータ制約を確認できません。現在のデータは変更されませんでした。",
            )
        })?;
        Ok(())
    }
    .await;
    let detach_result = sqlx::query("DETACH DATABASE uploaded")
        .execute(&mut *connection)
        .await;
    drop(connection);
    pool.close().await;
    match (copy_result, detach_result) {
        (Err(error), _) => Err(error),
        (Ok(()), Err(error)) => Err(AppError::context(
            "Failed to detach uploaded database",
            error,
        )),
        (Ok(()), Ok(_)) => Ok(()),
    }
}

async fn open_pool(path: &Path, read_only: bool) -> Result<SqlitePool, sqlx::Error> {
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(false)
                .read_only(read_only)
                .foreign_keys(true),
        )
        .await
}

async fn open_pool_create(path: &Path) -> AppResult<SqlitePool> {
    let url = format!("sqlite:{}", path.display());
    if !Sqlite::database_exists(&url).await.unwrap_or(false) {
        Sqlite::create_database(&url)
            .await
            .map_err(|error| AppError::context("Failed to create restore database", error))?;
    }
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::from_str(&url)
                .map_err(|error| AppError::context("Failed to configure restore database", error))?
                .foreign_keys(true),
        )
        .await
        .map_err(|error| AppError::context("Failed to open restore database", error))
}

async fn attach_database(
    connection: &mut SqliteConnection,
    path: &Path,
    schema: &str,
) -> AppResult<()> {
    let path = path
        .to_str()
        .ok_or_else(|| AppError::bad_request("一時ファイルのパスを処理できません。"))?;
    sqlx::query(AssertSqlSafe(format!("ATTACH DATABASE ? AS {schema}")))
        .bind(path)
        .execute(connection)
        .await
        .map_err(|error| AppError::context("Failed to attach restore database", error))?;
    Ok(())
}

async fn vacuum_into<'e, E>(executor: E, path: &Path, context: &str) -> AppResult<()>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let path = path.to_str().ok_or_else(|| {
        AppError::context(
            context,
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "path is not valid UTF-8"),
        )
    })?;
    sqlx::query("VACUUM INTO ?")
        .bind(path)
        .execute(executor)
        .await
        .map_err(|error| AppError::context(context, error))?;
    Ok(())
}

fn invalid_database(_error: impl std::fmt::Display) -> AppError {
    AppError::bad_request(
        "有効なSQLiteバックアップとして読み込めません。現在のデータは変更されませんでした。",
    )
}

pub fn temporary_path(directory: &Path, purpose: &str) -> AppResult<PathBuf> {
    Ok(directory.join(format!(
        "kakeibo-{purpose}-{}-{}.db",
        std::process::id(),
        unique_suffix()?
    )))
}

fn unique_suffix() -> AppResult<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .map_err(|error| AppError::context("Failed to create unique backup path", error))
}

pub fn remove_file(path: &Path) -> AppResult<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(AppError::context(
            "Failed to remove temporary database backup",
            error,
        )),
    }
}

fn create_private_directory(path: &Path) -> std::io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path)
}

#[cfg(unix)]
fn restrict_permissions(path: &Path, mode: u32) -> AppResult<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .map_err(|error| AppError::context("Failed to restrict safety backup permissions", error))
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path, _mode: u32) -> AppResult<()> {
    Ok(())
}

fn prune_safety_backups(config: &RestoreConfig) -> AppResult<()> {
    let mut paths = fs::read_dir(&config.safety_backup_directory)
        .map_err(|error| AppError::context("Failed to list safety backups", error))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.starts_with("kakeibo-before-restore-") && name.ends_with(".db")
                })
        })
        .collect::<Vec<_>>();
    paths.sort();
    let remove_count = paths.len().saturating_sub(config.safety_backup_generations);
    for path in paths.into_iter().take(remove_count) {
        remove_file(&path)?;
    }
    Ok(())
}
