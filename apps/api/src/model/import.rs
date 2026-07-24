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
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ImportResult {
    pub imported: usize,
    pub created_categories: Vec<String>,
    pub created_payment_methods: Vec<String>,
}
