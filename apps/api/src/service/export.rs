use crate::{
    model::export::{ExpenseCsvRecord, IncomeCsvRecord},
    repository::export::ExportRepository,
    utils::error::{AppError, AppResult},
};
use serde::Serialize;
const UTF8_BOM: &str = "\u{feff}";
#[derive(Clone)]
pub struct ExportService {
    repository: ExportRepository,
}
impl ExportService {
    pub fn new(repository: ExportRepository) -> Self {
        Self { repository }
    }
    pub async fn expenses_csv(&self) -> AppResult<String> {
        let records = self
            .repository
            .find_expenses()
            .await?
            .into_iter()
            .map(Into::into)
            .collect::<Vec<ExpenseCsvRecord>>();
        to_csv(&records)
    }
    pub async fn incomes_csv(&self) -> AppResult<String> {
        let records = self
            .repository
            .find_incomes()
            .await?
            .into_iter()
            .map(Into::into)
            .collect::<Vec<IncomeCsvRecord>>();
        to_csv(&records)
    }
}
fn to_csv<T: Serialize>(records: &[T]) -> AppResult<String> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    for record in records {
        writer
            .serialize(record)
            .map_err(|e| AppError::context("Failed to generate CSV", e))?;
    }
    let bytes = writer
        .into_inner()
        .map_err(|e| AppError::context("Failed to generate CSV", e))?;
    let text =
        String::from_utf8(bytes).map_err(|e| AppError::context("Failed to generate CSV", e))?;
    Ok(format!("{UTF8_BOM}{text}"))
}
