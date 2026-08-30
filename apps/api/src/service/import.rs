use crate::{
    model::{
        expenses::ExpenseUpsertRequest,
        import::{ExpenseCsvRow, ImportResult, IncomeCsvRow},
        incomes::IncomeUpsertRequest,
    },
    repository::import::ImportRepository,
    utils::error::{AppError, AppResult},
};
use chrono::NaiveDate;
use serde::de::DeserializeOwned;
const EXPENSE_HEADERS: &str = "日付,金額,カテゴリ,支払方法,メモ";
const INCOME_HEADERS: &str = "日付,金額,カテゴリ,メモ";
const MAX_REPORTED_ERRORS: usize = 3;
#[derive(Clone)]
pub struct ImportService {
    repository: ImportRepository,
}
impl ImportService {
    pub fn new(repository: ImportRepository) -> Self {
        Self { repository }
    }
    pub async fn import_expenses(&self, csv_text: &str) -> AppResult<ImportResult> {
        let rows = parse_csv::<ExpenseCsvRow>(csv_text, EXPENSE_HEADERS)?;
        validate_all(rows.iter().enumerate().map(|(index, row)| {
            let mut errors = Vec::new();
            validate_row(
                index + 2,
                &row.transaction_date,
                &row.amount,
                &[
                    ("カテゴリ", &row.category),
                    ("支払方法", &row.payment_method),
                ],
                &mut errors,
            );
            errors
        }))?;
        let mut tx = self.repository.begin().await?;
        let mut created_categories = Vec::new();
        let mut created_payment_methods = Vec::new();
        for row in &rows {
            let category = row.category.trim();
            let category_id = match self
                .repository
                .find_expense_category_id(&mut tx, category)
                .await?
            {
                Some(id) => id,
                None => {
                    let id = self
                        .repository
                        .create_expense_category(&mut tx, category)
                        .await?;
                    created_categories.push(category.to_owned());
                    id
                }
            };
            let payment_method = row.payment_method.trim();
            let payment_method_id = match self
                .repository
                .find_payment_method_id(&mut tx, payment_method)
                .await?
            {
                Some(id) => id,
                None => {
                    let id = self
                        .repository
                        .create_payment_method(&mut tx, payment_method)
                        .await?;
                    created_payment_methods.push(payment_method.to_owned());
                    id
                }
            };
            self.repository
                .insert_expense(
                    &mut tx,
                    &ExpenseUpsertRequest {
                        transaction_date: row.transaction_date.trim().to_owned(),
                        amount: row.amount.trim().to_owned(),
                        category_id,
                        payment_method_id,
                        recurring_expense_id: None,
                        description: row.description.clone(),
                    },
                )
                .await?;
        }
        tx.commit().await?;
        Ok(ImportResult {
            imported: rows.len(),
            created_categories,
            created_payment_methods,
        })
    }
    pub async fn import_incomes(&self, csv_text: &str) -> AppResult<ImportResult> {
        let rows = parse_csv::<IncomeCsvRow>(csv_text, INCOME_HEADERS)?;
        validate_all(rows.iter().enumerate().map(|(index, row)| {
            let mut errors = Vec::new();
            validate_row(
                index + 2,
                &row.transaction_date,
                &row.amount,
                &[("カテゴリ", &row.category)],
                &mut errors,
            );
            errors
        }))?;
        let mut tx = self.repository.begin().await?;
        let mut created_categories = Vec::new();
        for row in &rows {
            let category = row.category.trim();
            let category_id = match self
                .repository
                .find_income_category_id(&mut tx, category)
                .await?
            {
                Some(id) => id,
                None => {
                    let id = self
                        .repository
                        .create_income_category(&mut tx, category)
                        .await?;
                    created_categories.push(category.to_owned());
                    id
                }
            };
            self.repository
                .insert_income(
                    &mut tx,
                    &IncomeUpsertRequest {
                        transaction_date: row.transaction_date.trim().to_owned(),
                        amount: row.amount.trim().to_owned(),
                        category_id,
                        recurring_income_id: None,
                        description: row.description.clone(),
                    },
                )
                .await?;
        }
        tx.commit().await?;
        Ok(ImportResult {
            imported: rows.len(),
            created_categories,
            created_payment_methods: Vec::new(),
        })
    }
}
fn parse_csv<T: DeserializeOwned>(text: &str, expected_headers: &str) -> AppResult<Vec<T>> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    if text.trim().is_empty() {
        return Err(AppError::bad_request(&format!(
            "CSVファイルが空です。ヘッダー「{expected_headers}」に続けてデータ行を記述してください"
        )));
    }
    let mut reader = csv::Reader::from_reader(text.as_bytes());
    let headers = reader
        .headers()
        .map_err(|e| AppError::context("Failed to read CSV", e))?
        .clone();
    if headers.iter().collect::<Vec<_>>() != expected_headers.split(',').collect::<Vec<_>>() {
        return Err(AppError::bad_request(&format!(
            "CSVのヘッダーが正しくありません。「{expected_headers}」の形式のファイルを選択してください"
        )));
    }
    let mut rows = Vec::new();
    for (index, record) in reader.deserialize().enumerate() {
        let row = record.map_err(|error| {
            AppError::bad_request(&format!(
                "{}行目: 値の数や形式が正しくありません（{error}）",
                index + 2
            ))
        })?;
        rows.push(row);
    }
    Ok(rows)
}
fn validate_all(row_errors: impl Iterator<Item = Vec<String>>) -> AppResult<()> {
    let errors = row_errors.flatten().collect::<Vec<_>>();
    if errors.is_empty() {
        return Ok(());
    }
    let shown = errors
        .iter()
        .take(MAX_REPORTED_ERRORS)
        .cloned()
        .collect::<Vec<_>>()
        .join(" / ");
    let rest = errors.len().saturating_sub(MAX_REPORTED_ERRORS);
    let suffix = if rest > 0 {
        format!(" / ほか{rest}件のエラー")
    } else {
        String::new()
    };
    Err(AppError::bad_request(&format!("{shown}{suffix}")))
}
fn validate_row(
    row: usize,
    transaction_date: &str,
    amount: &str,
    required: &[(&str, &String)],
    errors: &mut Vec<String>,
) {
    if NaiveDate::parse_from_str(transaction_date.trim(), "%Y-%m-%d").is_err() {
        errors.push(format!(
            "{row}行目: 日付「{transaction_date}」はYYYY-MM-DD形式で指定してください"
        ));
    }
    if !matches!(amount.trim().parse::<u64>(), Ok(v) if v >= 1) {
        errors.push(format!(
            "{row}行目: 金額「{amount}」は1以上の整数で指定してください"
        ));
    }
    for (label, value) in required {
        if value.trim().is_empty() {
            errors.push(format!("{row}行目: {label}を指定してください"));
        }
    }
}
