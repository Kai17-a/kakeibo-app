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
