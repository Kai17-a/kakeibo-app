use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Debug, Clone, Deserialize)]
pub struct ExpenseCsvRow {
    #[serde(rename = "日付")]
    pub transaction_date: String,
    #[serde(rename = "金額")]
    pub amount: String,
    #[serde(rename = "カテゴリ")]
    pub category: String,
    #[serde(rename = "支払方法")]
    pub payment_method: String,
    #[serde(rename = "メモ")]
    pub description: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct IncomeCsvRow {
    #[serde(rename = "日付")]
    pub transaction_date: String,
    #[serde(rename = "金額")]
    pub amount: String,
    #[serde(rename = "カテゴリ")]
    pub category: String,
    #[serde(rename = "メモ")]
    pub description: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct RecurringExpenseCsvRow {
    #[serde(rename = "名称")]
    pub name: String,
    #[serde(rename = "金額")]
    pub amount: String,
    #[serde(rename = "通貨")]
    pub currency: String,
    #[serde(rename = "外貨金額")]
    pub foreign_amount: String,
    #[serde(rename = "支払日")]
    pub payment_day: String,
    #[serde(rename = "開始日")]
    pub start_date: String,
    #[serde(rename = "終了日")]
    pub end_date: Option<String>,
    #[serde(rename = "カテゴリ")]
    pub category: String,
    #[serde(rename = "支払方法")]
    pub payment_method: String,
    #[serde(rename = "金額変動")]
    pub is_variable: String,
    #[serde(rename = "備考")]
    pub description: Option<String>,
}
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ImportResult {
    pub imported: usize,
    pub created_categories: Vec<String>,
    pub created_payment_methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ExpenseImportPreview {
    pub rows: Vec<ExpensePreviewRow>,
    pub created_categories: Vec<String>,
    pub created_payment_methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ExpensePreviewRow {
    pub transaction_date: String,
    pub amount: String,
    pub category: String,
    pub category_is_new: bool,
    pub payment_method: String,
    pub payment_method_is_new: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct IncomeImportPreview {
    pub rows: Vec<IncomePreviewRow>,
    pub created_categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct IncomePreviewRow {
    pub transaction_date: String,
    pub amount: String,
    pub category: String,
    pub category_is_new: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RecurringExpenseImportPreview {
    pub rows: Vec<RecurringExpensePreviewRow>,
    pub created_categories: Vec<String>,
    pub created_payment_methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RecurringExpensePreviewRow {
    pub name: String,
    pub amount: String,
    pub currency: String,
    pub foreign_amount: String,
    pub payment_day: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub category: String,
    pub category_is_new: bool,
    pub payment_method: String,
    pub payment_method_is_new: bool,
    pub is_variable: String,
    pub description: Option<String>,
}
