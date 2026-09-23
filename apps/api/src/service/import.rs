use crate::{
    model::{
        expenses::ExpenseUpsertRequest,
        import::{
            ExpenseCsvRow, ExpenseImportPreview, ExpensePreviewRow, ImportResult, IncomeCsvRow,
            IncomeImportPreview, IncomePreviewRow, RecurringExpenseCsvRow,
            RecurringExpenseImportPreview, RecurringExpensePreviewRow,
        },
        incomes::IncomeUpsertRequest,
        recurring_expenses::RecurringExpenseUpsertRequest,
    },
    repository::import::ImportRepository,
    utils::error::{AppError, AppResult},
};
use chrono::NaiveDate;
use serde::de::DeserializeOwned;
const UTF8_BOM: &str = "\u{feff}";
const EXPENSE_HEADERS: &[&str] = &["日付", "金額", "カテゴリ", "支払方法", "メモ"];
const INCOME_HEADERS: &[&str] = &["日付", "金額", "カテゴリ", "メモ"];
const RECURRING_EXPENSE_HEADERS: &[&str] = &[
    "名称",
    "金額",
    "通貨",
    "外貨金額",
    "支払日",
    "開始日",
    "終了日",
    "カテゴリ",
    "支払方法",
    "金額変動",
    "備考",
];
const EXPENSE_SAMPLE_ROW: &[&str] = &["2026-01-15", "1200", "食費", "現金", "昼食"];
const INCOME_SAMPLE_ROW: &[&str] = &["2026-01-15", "300000", "給与", "1月分"];
const RECURRING_EXPENSE_SAMPLE_ROW: &[&str] = &[
    "家賃",
    "61100",
    "",
    "",
    "1",
    "2026-01-01",
    "",
    "住居費",
    "口座振替",
    "",
    "",
];
const MAX_REPORTED_ERRORS: usize = 3;
#[derive(Clone)]
pub struct ImportService {
    repository: ImportRepository,
}
impl ImportService {
    pub fn new(repository: ImportRepository) -> Self {
        Self { repository }
    }

    pub fn expense_sample_csv() -> String {
        sample_csv(EXPENSE_HEADERS, EXPENSE_SAMPLE_ROW)
    }

    pub fn income_sample_csv() -> String {
        sample_csv(INCOME_HEADERS, INCOME_SAMPLE_ROW)
    }

    pub fn recurring_expense_sample_csv() -> String {
        sample_csv(RECURRING_EXPENSE_HEADERS, RECURRING_EXPENSE_SAMPLE_ROW)
    }

    pub async fn preview_expenses(&self, csv_text: &str) -> AppResult<ExpenseImportPreview> {
        let rows = parse_csv::<ExpenseCsvRow>(csv_text, EXPENSE_HEADERS)?;
        validate_expenses(&rows)?;
        let mut tx = self.repository.begin().await?;
        let mut created_categories = Vec::new();
        let mut created_payment_methods = Vec::new();
        let mut preview_rows = Vec::with_capacity(rows.len());
        for row in &rows {
            let (category_id, category_is_new) = self
                .resolve_expense_category(&mut tx, &row.category, &mut created_categories)
                .await?;
            let (payment_method_id, payment_method_is_new) = self
                .resolve_payment_method(&mut tx, &row.payment_method, &mut created_payment_methods)
                .await?;
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
                        foreign_amount: None,
                        currency_code: None,
                        exchange_rate: None,
                        exchange_rate_date: None,
                    },
                )
                .await?;
            preview_rows.push(ExpensePreviewRow {
                transaction_date: row.transaction_date.trim().to_owned(),
                amount: row.amount.trim().to_owned(),
                category: row.category.trim().to_owned(),
                category_is_new,
                payment_method: row.payment_method.trim().to_owned(),
                payment_method_is_new,
                description: row.description.clone(),
            });
        }
        tx.rollback().await?;
        Ok(ExpenseImportPreview {
            rows: preview_rows,
            created_categories,
            created_payment_methods,
        })
    }

    pub async fn preview_incomes(&self, csv_text: &str) -> AppResult<IncomeImportPreview> {
        let rows = parse_csv::<IncomeCsvRow>(csv_text, INCOME_HEADERS)?;
        validate_incomes(&rows)?;
        let mut tx = self.repository.begin().await?;
        let mut created_categories = Vec::new();
        let mut preview_rows = Vec::with_capacity(rows.len());
        for row in &rows {
            let (category_id, category_is_new) = self
                .resolve_income_category(&mut tx, &row.category, &mut created_categories)
                .await?;
            self.repository
                .insert_income(
                    &mut tx,
                    &IncomeUpsertRequest {
                        transaction_date: row.transaction_date.trim().to_owned(),
                        amount: row.amount.trim().to_owned(),
                        category_id,
                        payment_method_id: None,
                        recurring_income_id: None,
                        description: row.description.clone(),
                    },
                )
                .await?;
            preview_rows.push(IncomePreviewRow {
                transaction_date: row.transaction_date.trim().to_owned(),
                amount: row.amount.trim().to_owned(),
                category: row.category.trim().to_owned(),
                category_is_new,
                description: row.description.clone(),
            });
        }
        tx.rollback().await?;
        Ok(IncomeImportPreview {
            rows: preview_rows,
            created_categories,
        })
    }

    pub async fn preview_recurring_expenses(
        &self,
        csv_text: &str,
    ) -> AppResult<RecurringExpenseImportPreview> {
        let rows = parse_csv::<RecurringExpenseCsvRow>(csv_text, RECURRING_EXPENSE_HEADERS)?;
        validate_recurring_expenses(&rows)?;
        let mut tx = self.repository.begin().await?;
        let mut created_categories = Vec::new();
        let mut created_payment_methods = Vec::new();
        let mut preview_rows = Vec::with_capacity(rows.len());
        for row in &rows {
            let (category_id, category_is_new) = self
                .resolve_expense_category(&mut tx, &row.category, &mut created_categories)
                .await?;
            let (payment_method_id, payment_method_is_new) = self
                .resolve_payment_method(&mut tx, &row.payment_method, &mut created_payment_methods)
                .await?;
            self.repository
                .insert_recurring_expense(
                    &mut tx,
                    &recurring_expense_request(row, category_id, payment_method_id),
                )
                .await?;
            preview_rows.push(RecurringExpensePreviewRow {
                name: row.name.trim().to_owned(),
                amount: row.amount.trim().to_owned(),
                currency: row.currency.trim().to_owned(),
                foreign_amount: row.foreign_amount.trim().to_owned(),
                payment_day: row.payment_day.trim().to_owned(),
                start_date: row.start_date.trim().to_owned(),
                end_date: row
                    .end_date
                    .clone()
                    .filter(|value| !value.trim().is_empty()),
                category: row.category.trim().to_owned(),
                category_is_new,
                payment_method: row.payment_method.trim().to_owned(),
                payment_method_is_new,
                is_variable: row.is_variable.trim().to_owned(),
                description: row.description.clone(),
            });
        }
        tx.rollback().await?;
        Ok(RecurringExpenseImportPreview {
            rows: preview_rows,
            created_categories,
            created_payment_methods,
        })
    }

    pub async fn import_expenses(&self, csv_text: &str) -> AppResult<ImportResult> {
        let rows = parse_csv::<ExpenseCsvRow>(csv_text, EXPENSE_HEADERS)?;
        validate_expenses(&rows)?;
        let mut tx = self.repository.begin().await?;
        let mut created_categories = Vec::new();
        let mut created_payment_methods = Vec::new();
        for row in &rows {
            let (category_id, _) = self
                .resolve_expense_category(&mut tx, &row.category, &mut created_categories)
                .await?;
            let (payment_method_id, _) = self
                .resolve_payment_method(&mut tx, &row.payment_method, &mut created_payment_methods)
                .await?;
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
                        foreign_amount: None,
                        currency_code: None,
                        exchange_rate: None,
                        exchange_rate_date: None,
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
        validate_incomes(&rows)?;
        let mut tx = self.repository.begin().await?;
        let mut created_categories = Vec::new();
        for row in &rows {
            let (category_id, _) = self
                .resolve_income_category(&mut tx, &row.category, &mut created_categories)
                .await?;
            self.repository
                .insert_income(
                    &mut tx,
                    &IncomeUpsertRequest {
                        transaction_date: row.transaction_date.trim().to_owned(),
                        amount: row.amount.trim().to_owned(),
                        category_id,
                        payment_method_id: None,
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

    pub async fn import_recurring_expenses(&self, csv_text: &str) -> AppResult<ImportResult> {
        let rows = parse_csv::<RecurringExpenseCsvRow>(csv_text, RECURRING_EXPENSE_HEADERS)?;
        validate_recurring_expenses(&rows)?;

        let mut tx = self.repository.begin().await?;
        let mut created_categories = Vec::new();
        let mut created_payment_methods = Vec::new();
        for row in &rows {
            let (category_id, _) = self
                .resolve_expense_category(&mut tx, &row.category, &mut created_categories)
                .await?;
            let (payment_method_id, _) = self
                .resolve_payment_method(&mut tx, &row.payment_method, &mut created_payment_methods)
                .await?;
            self.repository
                .insert_recurring_expense(
                    &mut tx,
                    &recurring_expense_request(row, category_id, payment_method_id),
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

    async fn resolve_expense_category(
        &self,
        tx: &mut sqlx::Transaction<'static, sqlx::Sqlite>,
        name: &str,
        created: &mut Vec<String>,
    ) -> AppResult<(String, bool)> {
        let name = name.trim();
        if let Some(id) = self.repository.find_expense_category_id(tx, name).await? {
            return Ok((id, false));
        }
        let id = self.repository.create_expense_category(tx, name).await?;
        created.push(name.to_owned());
        Ok((id, true))
    }

    async fn resolve_income_category(
        &self,
        tx: &mut sqlx::Transaction<'static, sqlx::Sqlite>,
        name: &str,
        created: &mut Vec<String>,
    ) -> AppResult<(String, bool)> {
        let name = name.trim();
        if let Some(id) = self.repository.find_income_category_id(tx, name).await? {
            return Ok((id, false));
        }
        let id = self.repository.create_income_category(tx, name).await?;
        created.push(name.to_owned());
        Ok((id, true))
    }

    async fn resolve_payment_method(
        &self,
        tx: &mut sqlx::Transaction<'static, sqlx::Sqlite>,
        name: &str,
        created: &mut Vec<String>,
    ) -> AppResult<(String, bool)> {
        let name = name.trim();
        if let Some(id) = self.repository.find_payment_method_id(tx, name).await? {
            return Ok((id, false));
        }
        let id = self.repository.create_payment_method(tx, name).await?;
        created.push(name.to_owned());
        Ok((id, true))
    }
}

fn validate_expenses(rows: &[ExpenseCsvRow]) -> AppResult<()> {
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
    }))
}

fn validate_incomes(rows: &[IncomeCsvRow]) -> AppResult<()> {
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
    }))
}

fn validate_recurring_expenses(rows: &[RecurringExpenseCsvRow]) -> AppResult<()> {
    validate_all(rows.iter().enumerate().map(|(index, row)| {
        let row_number = index + 2;
        let mut errors = Vec::new();
        validate_required(
            row_number,
            &[
                ("名称", &row.name),
                ("支払日", &row.payment_day),
                ("開始日", &row.start_date),
                ("カテゴリ", &row.category),
                ("支払方法", &row.payment_method),
            ],
            &mut errors,
        );
        if NaiveDate::parse_from_str(row.start_date.trim(), "%Y-%m-%d").is_err() {
            errors.push(format!(
                "{row_number}行目: 開始日「{}」はYYYY-MM-DD形式で指定してください",
                row.start_date
            ));
        }
        if !row.end_date.as_deref().unwrap_or("").trim().is_empty()
            && NaiveDate::parse_from_str(row.end_date.as_deref().unwrap().trim(), "%Y-%m-%d")
                .is_err()
        {
            errors.push(format!(
                "{row_number}行目: 終了日「{}」はYYYY-MM-DD形式で指定してください",
                row.end_date.as_deref().unwrap_or("")
            ));
        }
        let payment_day = row.payment_day.trim().parse::<i64>().ok();
        if !payment_day.is_some_and(|day| (1..=31).contains(&day)) {
            errors.push(format!(
                "{row_number}行目: 支払日「{}」は1〜31の整数で指定してください",
                row.payment_day
            ));
        }
        let currency = row.currency.trim();
        if currency.is_empty() {
            if !is_positive_integer(&row.amount) {
                errors.push(format!(
                    "{row_number}行目: 金額「{}」は1以上の整数で指定してください",
                    row.amount
                ));
            }
            if !row.foreign_amount.trim().is_empty() {
                errors.push(format!(
                    "{row_number}行目: 円建てでは外貨金額を空欄にしてください"
                ));
            }
        } else if currency.eq_ignore_ascii_case("USD") {
            if !is_positive_number(&row.foreign_amount) {
                errors.push(format!(
                    "{row_number}行目: 外貨金額「{}」は正の数値で指定してください",
                    row.foreign_amount
                ));
            }
        } else {
            errors.push(format!(
                "{row_number}行目: 通貨「{}」はUSDまたは空欄で指定してください",
                row.currency
            ));
        }
        if !matches!(
            row.is_variable.trim().to_ascii_lowercase().as_str(),
            "" | "false" | "0" | "true" | "1"
        ) {
            errors.push(format!(
                "{row_number}行目: 金額変動「{}」はtrue/falseまたは1/0で指定してください",
                row.is_variable
            ));
        }
        errors
    }))
}

fn recurring_expense_request(
    row: &RecurringExpenseCsvRow,
    category_id: String,
    payment_method_id: String,
) -> RecurringExpenseUpsertRequest {
    let currency = row.currency.trim();
    let usd = currency.eq_ignore_ascii_case("USD");
    RecurringExpenseUpsertRequest {
        name: row.name.trim().to_owned(),
        amount: if usd {
            row.foreign_amount.trim()
        } else {
            row.amount.trim()
        }
        .to_owned(),
        payment_day: row
            .payment_day
            .trim()
            .parse()
            .expect("validated payment day"),
        start_date: row.start_date.trim().to_owned(),
        end_date: row
            .end_date
            .clone()
            .filter(|value| !value.trim().is_empty()),
        category_id,
        payment_method_id,
        is_active: true,
        is_variable: matches!(
            row.is_variable.trim().to_ascii_lowercase().as_str(),
            "true" | "1"
        ),
        description: row.description.clone(),
        foreign_amount: usd.then(|| row.foreign_amount.trim().to_owned()),
        currency_code: usd.then(|| "USD".to_owned()),
        exchange_rate: None,
        sync_future_transactions: false,
    }
}
fn sample_csv(headers: &[&str], example_row: &[&str]) -> String {
    format!(
        "{UTF8_BOM}{}\n{}\n",
        headers.join(","),
        example_row.join(",")
    )
}

fn parse_csv<T: DeserializeOwned>(text: &str, expected_headers: &[&str]) -> AppResult<Vec<T>> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let expected_headers_text = expected_headers.join(",");
    if text.trim().is_empty() {
        return Err(AppError::bad_request(&format!(
            "CSVファイルが空です。ヘッダー「{expected_headers_text}」に続けてデータ行を記述してください"
        )));
    }
    let mut reader = csv::Reader::from_reader(text.as_bytes());
    let headers = reader
        .headers()
        .map_err(|e| AppError::context("Failed to read CSV", e))?
        .clone();
    if headers.iter().collect::<Vec<_>>() != expected_headers {
        return Err(AppError::bad_request(&format!(
            "CSVのヘッダーが正しくありません。「{expected_headers_text}」の形式のファイルを選択してください"
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

fn validate_required(row: usize, required: &[(&str, &String)], errors: &mut Vec<String>) {
    for (label, value) in required {
        if value.trim().is_empty() {
            errors.push(format!("{row}行目: {label}を指定してください"));
        }
    }
}

fn is_positive_integer(value: &str) -> bool {
    matches!(value.trim().parse::<u64>(), Ok(value) if value >= 1)
}

fn is_positive_number(value: &str) -> bool {
    value
        .trim()
        .parse::<f64>()
        .is_ok_and(|value| value.is_finite() && value > 0.0)
}
