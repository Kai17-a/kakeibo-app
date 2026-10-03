use crate::{
    model::{
        expenses::ExpenseUpsertRequest,
        import::{
            ExpenseCsvRow, ExpenseImportPreview, ExpensePreviewRow, ImportResult, IncomeCsvRow,
            IncomeImportPreview, IncomePreviewRow, RecurringExpenseCsvRow,
            RecurringExpenseImportPreview, RecurringExpensePreviewRow, TransferCsvRow,
            TransferImportPreview, TransferPreviewRow, VariableExpenseCsvRow,
            VariableExpenseDefinition, VariableExpenseImportPreview, VariableExpensePreviewRow,
        },
        incomes::IncomeUpsertRequest,
        recurring_expenses::RecurringExpenseUpsertRequest,
        transfers::TransferUpsertRequest,
    },
    repository::import::ImportRepository,
    utils::error::{AppError, AppResult},
};
use chrono::{Datelike, NaiveDate};
use serde::de::DeserializeOwned;
use std::collections::HashSet;
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
const VARIABLE_EXPENSE_HEADERS: &[&str] = &["年月", "名称", "金額", "メモ"];
const TRANSFER_HEADERS: &[&str] = &["日付", "金額", "移動元", "移動先", "メモ"];
const EXPENSE_SAMPLE_ROW: &[&str] = &["2026-01-15", "1200", "食費", "現金", "昼食"];
const INCOME_SAMPLE_ROW: &[&str] = &["2026-01-15", "300000", "給与", "1月分"];
const RECURRING_EXPENSE_SAMPLE_ROW: &[&str] = &[
    "家賃",
    "61100",
    "",
    "",
    "月末",
    "2026-01-01",
    "",
    "住居費",
    "口座振替",
    "",
    "",
];
const VARIABLE_RECURRING_EXPENSE_SAMPLE_ROW: &[&str] = &[
    "電気代",
    "",
    "",
    "",
    "15",
    "2026-01-01",
    "",
    "水道光熱費",
    "口座振替",
    "true",
    "金額は毎月入力",
];
const VARIABLE_EXPENSE_SAMPLE_ROW: &[&str] = &["2026-01", "電気代", "12000", "1月分"];
const TRANSFER_SAMPLE_ROW: &[&str] = &[
    "2026-01-15",
    "30000",
    "クレジットカード",
    "NISA口座",
    "積立",
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
        format!(
            "{UTF8_BOM}{}\n{}\n{}\n",
            RECURRING_EXPENSE_HEADERS.join(","),
            RECURRING_EXPENSE_SAMPLE_ROW.join(","),
            VARIABLE_RECURRING_EXPENSE_SAMPLE_ROW.join(",")
        )
    }

    pub fn variable_expense_sample_csv() -> String {
        sample_csv(VARIABLE_EXPENSE_HEADERS, VARIABLE_EXPENSE_SAMPLE_ROW)
    }
    pub fn transfer_sample_csv() -> String {
        sample_csv(TRANSFER_HEADERS, TRANSFER_SAMPLE_ROW)
    }

    pub async fn preview_transfers(&self, csv_text: &str) -> AppResult<TransferImportPreview> {
        let rows = parse_csv::<TransferCsvRow>(csv_text, TRANSFER_HEADERS)?;
        validate_transfers(&rows)?;
        let mut tx = self.repository.begin().await?;
        let prepared = self.prepare_transfers(&mut tx, &rows).await?;
        let preview = prepared
            .into_iter()
            .map(|(row, _)| TransferPreviewRow {
                transaction_date: row.transaction_date.trim().to_owned(),
                amount: row.amount.trim().to_owned(),
                from_payment_method: row.from_payment_method.trim().to_owned(),
                to_payment_method: row.to_payment_method.trim().to_owned(),
                description: row.description.clone(),
            })
            .collect();
        tx.rollback().await?;
        Ok(TransferImportPreview { rows: preview })
    }

    pub async fn import_transfers(&self, csv_text: &str) -> AppResult<ImportResult> {
        let rows = parse_csv::<TransferCsvRow>(csv_text, TRANSFER_HEADERS)?;
        validate_transfers(&rows)?;
        let mut tx = self.repository.begin().await?;
        let prepared = self.prepare_transfers(&mut tx, &rows).await?;
        for (_, request) in &prepared {
            self.repository.insert_transfer(&mut tx, request).await?;
        }
        tx.commit().await?;
        Ok(ImportResult {
            imported: prepared.len(),
            created_categories: Vec::new(),
            created_payment_methods: Vec::new(),
        })
    }

    async fn prepare_transfers(
        &self,
        tx: &mut sqlx::Transaction<'static, sqlx::Sqlite>,
        rows: &[TransferCsvRow],
    ) -> AppResult<Vec<(TransferCsvRow, TransferUpsertRequest)>> {
        let mut prepared = Vec::with_capacity(rows.len());
        let mut errors = Vec::new();
        for (index, row) in rows.iter().enumerate() {
            let line = index + 2;
            let from_name = row.from_payment_method.trim();
            let to_name = row.to_payment_method.trim();
            let from = self
                .repository
                .find_payment_method_ids(tx, from_name)
                .await?;
            let to = self.repository.find_payment_method_ids(tx, to_name).await?;
            if from.is_empty() {
                errors.push(format!("{line}行目: 移動元「{from_name}」が見つかりません"));
            } else if from.len() > 1 {
                errors.push(format!(
                    "{line}行目: 移動元「{from_name}」が複数あり特定できません"
                ));
            }
            if to.is_empty() {
                errors.push(format!("{line}行目: 移動先「{to_name}」が見つかりません"));
            } else if to.len() > 1 {
                errors.push(format!(
                    "{line}行目: 移動先「{to_name}」が複数あり特定できません"
                ));
            }
            if from_name == to_name {
                errors.push(format!(
                    "{line}行目: 移動元と移動先は異なる支払方法を指定してください"
                ));
            }
            if let ([from_id], [to_id]) = (from.as_slice(), to.as_slice())
                && from_id != to_id
            {
                prepared.push((
                    row.clone(),
                    TransferUpsertRequest {
                        transaction_date: row.transaction_date.trim().to_owned(),
                        amount: row.amount.trim().to_owned(),
                        from_payment_method_id: from_id.clone(),
                        to_payment_method_id: to_id.clone(),
                        description: row.description.clone(),
                    },
                ));
            }
        }
        validate_all(std::iter::once(errors))?;
        Ok(prepared)
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
                payment_day: parse_payment_day(&row.payment_day)
                    .expect("validated payment day")
                    .to_string(),
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

    pub async fn preview_variable_expenses(
        &self,
        csv_text: &str,
    ) -> AppResult<VariableExpenseImportPreview> {
        let rows = parse_csv::<VariableExpenseCsvRow>(csv_text, VARIABLE_EXPENSE_HEADERS)?;
        validate_variable_expenses(&rows)?;
        let mut tx = self.repository.begin().await?;
        let prepared = self.prepare_variable_expenses(&mut tx, &rows).await?;
        let preview_rows = prepared
            .into_iter()
            .map(|row| VariableExpensePreviewRow {
                year_month: row.year_month,
                name: row.definition.name,
                transaction_date: row.transaction_date,
                amount: row.amount,
                category: row.definition.category,
                payment_method: row.definition.payment_method,
                description: row.description,
            })
            .collect();
        tx.rollback().await?;
        Ok(VariableExpenseImportPreview {
            rows: preview_rows,
            created_categories: Vec::new(),
            created_payment_methods: Vec::new(),
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

    pub async fn import_variable_expenses(&self, csv_text: &str) -> AppResult<ImportResult> {
        let rows = parse_csv::<VariableExpenseCsvRow>(csv_text, VARIABLE_EXPENSE_HEADERS)?;
        validate_variable_expenses(&rows)?;
        let mut tx = self.repository.begin().await?;
        let prepared = self.prepare_variable_expenses(&mut tx, &rows).await?;
        for row in &prepared {
            self.repository
                .insert_expense(
                    &mut tx,
                    &ExpenseUpsertRequest {
                        transaction_date: row.transaction_date.clone(),
                        amount: row.amount.clone(),
                        category_id: row.definition.category_id.clone(),
                        payment_method_id: row.definition.payment_method_id.clone(),
                        recurring_expense_id: Some(row.definition.id.clone()),
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
            imported: prepared.len(),
            created_categories: Vec::new(),
            created_payment_methods: Vec::new(),
        })
    }

    async fn prepare_variable_expenses(
        &self,
        tx: &mut sqlx::Transaction<'static, sqlx::Sqlite>,
        rows: &[VariableExpenseCsvRow],
    ) -> AppResult<Vec<PreparedVariableExpense>> {
        let mut prepared = Vec::with_capacity(rows.len());
        let mut errors = Vec::new();
        let mut seen = HashSet::new();
        for (index, row) in rows.iter().enumerate() {
            let row_number = index + 2;
            let name = row.name.trim();
            let matches = self
                .repository
                .find_recurring_expenses_by_name(tx, name)
                .await?;
            let definition = match matches.as_slice() {
                [] => {
                    errors.push(format!(
                        "{row_number}行目: 名称「{name}」の定期支出が見つかりません"
                    ));
                    continue;
                }
                [definition] if !definition.is_variable => {
                    errors.push(format!(
                        "{row_number}行目: 名称「{name}」は準固定費ではなく固定費です"
                    ));
                    continue;
                }
                [definition] => definition.clone(),
                _ => {
                    errors.push(format!(
                        "{row_number}行目: 名称「{name}」に一致する定期支出が複数あり特定できません"
                    ));
                    continue;
                }
            };
            if !(1..=31).contains(&definition.payment_day) {
                errors.push(format!(
                    "{row_number}行目: 名称「{name}」の支払日「{}」が1〜31の範囲外のため計上日を決められません",
                    definition.payment_day
                ));
                continue;
            }
            let year_month = row.year_month.trim().to_owned();
            let key = (definition.id.clone(), year_month.clone());
            if !seen.insert(key) {
                errors.push(format!(
                    "{row_number}行目: 名称「{name}」の{year_month}はCSV内で重複しています"
                ));
                continue;
            }
            if self
                .repository
                .has_variable_expense_for_month(tx, &definition.id, &year_month)
                .await?
            {
                errors.push(format!(
                    "{row_number}行目: 名称「{name}」の{year_month}は登録済みです"
                ));
                continue;
            }
            let transaction_date = payment_date(&year_month, definition.payment_day);
            let description = row
                .description
                .as_ref()
                .filter(|value| !value.trim().is_empty())
                .cloned()
                .or_else(|| Some(definition.name.clone()));
            prepared.push(PreparedVariableExpense {
                year_month,
                transaction_date,
                amount: row.amount.trim().to_owned(),
                description,
                definition,
            });
        }
        validate_all(std::iter::once(errors))?;
        Ok(prepared)
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
        let payment_day = parse_payment_day(&row.payment_day);
        if !payment_day.is_some_and(|day| (1..=31).contains(&day)) {
            errors.push(format!(
                "{row_number}行目: 支払日「{}」は1〜31の整数または月末で指定してください",
                row.payment_day
            ));
        }
        let currency = row.currency.trim();
        let is_variable = matches!(
            row.is_variable.trim().to_ascii_lowercase().as_str(),
            "true" | "1"
        );
        if currency.is_empty() {
            if (!is_variable || !row.amount.trim().is_empty()) && !is_positive_integer(&row.amount)
            {
                errors.push(format!(
                    "{row_number}行目: 固定費の金額、または準固定費で入力する金額「{}」は1以上の整数で指定してください",
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

fn validate_variable_expenses(rows: &[VariableExpenseCsvRow]) -> AppResult<()> {
    validate_all(rows.iter().enumerate().map(|(index, row)| {
        let row_number = index + 2;
        let mut errors = Vec::new();
        validate_required(row_number, &[("名称", &row.name)], &mut errors);
        if parse_year_month(row.year_month.trim()).is_none() {
            errors.push(format!(
                "{row_number}行目: 年月「{}」はYYYY-MM形式で指定してください",
                row.year_month
            ));
        }
        if !is_positive_integer(&row.amount) {
            errors.push(format!(
                "{row_number}行目: 金額「{}」は1以上の整数で指定してください",
                row.amount
            ));
        }
        errors
    }))
}

fn validate_transfers(rows: &[TransferCsvRow]) -> AppResult<()> {
    validate_all(rows.iter().enumerate().map(|(index, row)| {
        let mut errors = Vec::new();
        validate_row(
            index + 2,
            &row.transaction_date,
            &row.amount,
            &[
                ("移動元", &row.from_payment_method),
                ("移動先", &row.to_payment_method),
            ],
            &mut errors,
        );
        if matches!(row.amount.trim().parse::<u64>(), Ok(value) if value >= 1)
            && !crate::service::transfers::is_valid_amount(row.amount.trim())
        {
            errors.push(format!(
                "{}行目: 金額「{}」はi64の範囲内で指定してください",
                index + 2,
                row.amount
            ));
        }
        errors
    }))
}

#[derive(Debug)]
struct PreparedVariableExpense {
    year_month: String,
    transaction_date: String,
    amount: String,
    description: Option<String>,
    definition: VariableExpenseDefinition,
}

fn parse_year_month(value: &str) -> Option<NaiveDate> {
    let (year, month) = value.split_once('-')?;
    if year.len() != 4 || month.len() != 2 {
        return None;
    }
    if !year
        .bytes()
        .chain(month.bytes())
        .all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let year = year.parse::<i32>().ok().filter(|year| *year >= 1)?;
    NaiveDate::from_ymd_opt(year, month.parse().ok()?, 1)
}

fn payment_date(year_month: &str, payment_day: i64) -> String {
    let first = parse_year_month(year_month).expect("validated year-month");
    let (next_year, next_month) = if first.month() == 12 {
        (first.year() + 1, 1)
    } else {
        (first.year(), first.month() + 1)
    };
    let last_day = NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .expect("valid next month")
        .pred_opt()
        .expect("month has a previous day")
        .day();
    first
        .with_day(u32::try_from(payment_day).map_or(last_day, |day| day.clamp(1, last_day)))
        .expect("valid clamped payment day")
        .to_string()
}

fn parse_payment_day(value: &str) -> Option<i64> {
    let value = value.trim();
    if value == "月末" {
        Some(31)
    } else {
        value.parse().ok()
    }
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
        amount: Some(
            if usd {
                row.foreign_amount.trim()
            } else {
                row.amount.trim()
            }
            .to_owned(),
        )
        .filter(|amount| !amount.is_empty()),
        payment_day: parse_payment_day(&row.payment_day).expect("validated payment day"),
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
