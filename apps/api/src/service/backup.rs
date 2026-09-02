use crate::utils::error::{AppError, AppResult};
use sqlx::SqlitePool;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Backup {
    pub bytes: Vec<u8>,
    pub date: String,
}

#[derive(Clone)]
pub struct BackupService {
    pool: SqlitePool,
}

impl BackupService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self) -> AppResult<Backup> {
        let date = sqlx::query_scalar::<_, String>("SELECT strftime('%Y%m%d', 'now', 'localtime')")
            .fetch_one(&self.pool)
            .await?;
        let path = temporary_path()?;

        let result = self.create_file(&path).await;
        let cleanup_result = remove_file(&path);

        match (result, cleanup_result) {
            (Ok(bytes), Ok(())) => Ok(Backup { bytes, date }),
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(error),
        }
    }

    async fn create_file(&self, path: &Path) -> AppResult<Vec<u8>> {
        let path = path.to_str().ok_or_else(|| {
            AppError::context(
                "Failed to create database backup",
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "temporary path is not valid UTF-8",
                ),
            )
        })?;

        sqlx::query("VACUUM INTO ?")
            .bind(path)
            .execute(&self.pool)
            .await
            .map_err(|error| AppError::context("Failed to create database backup", error))?;

        fs::read(path).map_err(|error| AppError::context("Failed to read database backup", error))
    }
}

fn temporary_path() -> AppResult<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| AppError::context("Failed to create database backup", error))?
        .as_nanos();
    Ok(std::env::temp_dir().join(format!(
        "kakeibo-backup-{}-{timestamp}.db",
        std::process::id()
    )))
}

fn remove_file(path: &Path) -> AppResult<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(AppError::context(
            "Failed to remove temporary database backup",
            error,
        )),
    }
}
