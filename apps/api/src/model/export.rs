use crate::database::models::export::{ExpenseExportRow, IncomeExportRow};
use serde::Serialize;
#[derive(Debug, Clone, Serialize)]
pub struct ExpenseCsvRecord {
    #[serde(rename = "日付")]
    pub transaction_date: String,
    #[serde(rename = "金額")]
    pub amount: String,
    #[serde(rename = "カテゴリ")]
    pub category: String,
    #[serde(rename = "支払方法")]
    pub payment_method: String,
    #[serde(rename = "メモ")]
    pub description: String,
}
impl From<ExpenseExportRow> for ExpenseCsvRecord {
    fn from(v: ExpenseExportRow) -> Self {
        Self {
            transaction_date: v.transaction_date,
            amount: v.amount,
            category: v.category_name,
            payment_method: v.payment_method_name,
            description: v.description.unwrap_or_default(),
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct IncomeCsvRecord {
    #[serde(rename = "日付")]
    pub transaction_date: String,
    #[serde(rename = "金額")]
    pub amount: String,
    #[serde(rename = "カテゴリ")]
    pub category: String,
    #[serde(rename = "メモ")]
    pub description: String,
}
impl From<IncomeExportRow> for IncomeCsvRecord {
    fn from(v: IncomeExportRow) -> Self {
        Self {
            transaction_date: v.transaction_date,
            amount: v.amount,
            category: v.category_name,
            description: v.description.unwrap_or_default(),
        }
    }
}
