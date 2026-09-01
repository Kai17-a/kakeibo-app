use axum::Router;
use serde_json::json;
use utoipa::{Modify, OpenApi, openapi::path::Operation};
use utoipa_redoc::{Redoc, Servable};

use crate::{
    handler::{
        budgets, expense_categories, expenses, export, health, import, income_categories, incomes,
        payment_methods, recurring_expenses, recurring_incomes, webhook_urls,
    },
    model::budgets::{Budget, BudgetUpsertRequest},
    model::expense_categories::{
        ExpenseCategory, ExpenseCategoryListResponse, ExpenseCategoryPagination,
        ExpenseCategorySortBy, ExpenseCategorySortOrder, ExpenseCategoryUpsertRequest,
    },
    model::expenses::{Expense, ExpenseUpsertRequest},
    model::health::Health,
    model::import::ImportResult,
    model::income_categories::{
        IncomeCategory, IncomeCategoryListResponse, IncomeCategoryPagination, IncomeCategorySortBy,
        IncomeCategorySortOrder, IncomeCategoryUpsertRequest,
    },
    model::incomes::{
        Income, IncomeListResponse, IncomeQuery, IncomeSortBy, IncomeUpsertRequest, Pagination,
        SortOrder,
    },
    model::payment_methods::{
        PaymentMethod, PaymentMethodListResponse, PaymentMethodPagination, PaymentMethodSortBy,
        PaymentMethodSortOrder, PaymentMethodUpsertRequest,
    },
    model::recurring_expenses::{RecurringExpense, RecurringExpenseUpsertRequest},
    model::recurring_incomes::{RecurringIncome, RecurringIncomeUpsertRequest},
    model::webhook_urls::{WebhookUrl, WebhookUrlUpsertRequest},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        budgets::list,
        budgets::get,
        budgets::create,
        budgets::update,
        budgets::delete,
        expense_categories::list,
        expense_categories::get,
        expense_categories::create,
        expense_categories::update,
        expense_categories::delete,
        expenses::list,
        expenses::get,
        expenses::create,
        expenses::update,
        expenses::delete,
        export::expenses,
        export::incomes,
        import::expenses,
        import::incomes,
        payment_methods::list,
        payment_methods::get,
        payment_methods::create,
        payment_methods::update,
        payment_methods::delete,
        recurring_expenses::list,
        recurring_expenses::get,
        recurring_expenses::create,
        recurring_expenses::update,
        recurring_expenses::delete,
        recurring_incomes::list,
        recurring_incomes::get,
        recurring_incomes::create,
        recurring_incomes::update,
        recurring_incomes::delete,
        income_categories::list,
        income_categories::get,
        income_categories::create,
        income_categories::update,
        income_categories::delete,
        incomes::list,
        incomes::get,
        incomes::create,
        incomes::update,
        incomes::delete,
        webhook_urls::list,
        webhook_urls::get,
        webhook_urls::create,
        webhook_urls::update,
        webhook_urls::delete,
        health::get
    ),
    components(schemas(
        Budget,
        BudgetUpsertRequest,
        ExpenseCategory,
        ExpenseCategoryUpsertRequest,
        ExpenseCategoryListResponse,
        ExpenseCategoryPagination,
        ExpenseCategorySortBy,
        ExpenseCategorySortOrder,
        Expense,
        ExpenseUpsertRequest,
        ImportResult,
        PaymentMethod,
        PaymentMethodUpsertRequest,
        PaymentMethodListResponse,
        PaymentMethodPagination,
        PaymentMethodSortBy,
        PaymentMethodSortOrder,
        RecurringExpense,
        RecurringExpenseUpsertRequest,
        RecurringIncome,
        RecurringIncomeUpsertRequest,
        IncomeCategory,
        IncomeCategoryUpsertRequest,
        IncomeCategoryListResponse,
        IncomeCategoryPagination,
        IncomeCategorySortBy,
        IncomeCategorySortOrder,
        Income,
        IncomeUpsertRequest,
        IncomeQuery,
        IncomeListResponse,
        Pagination,
        IncomeSortBy,
        SortOrder,
        WebhookUrl,
        WebhookUrlUpsertRequest,
        Health
    )),
    modifiers(&Documentation),
    info(
        title = "Kakeibo API",
        version = "0.1.0"
    ),
    tags(
        (name = "収入", description = "収入の登録・参照・更新・削除"),
        (name = "収入カテゴリ", description = "収入カテゴリの管理"),
        (name = "支出", description = "支出の登録・参照・更新・削除"),
        (name = "支出カテゴリ", description = "支出カテゴリの管理"),
        (name = "エクスポート", description = "収支データのCSVエクスポート"),
        (name = "インポート", description = "収支データのCSVインポート"),
        (name = "支払方法", description = "支払方法の管理"),
        (name = "定期支出", description = "定期支出の管理"),
        (name = "定期収入", description = "定期収入の管理"),
        (name = "Webhook", description = "Webhook通知先URLの管理"),
        (name = "予算", description = "カテゴリ別月額予算の管理"),
        (name = "システム", description = "稼働状態の確認")
    )
)]
struct ApiDoc;

struct Documentation;

impl Modify for Documentation {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        for (path, item) in &mut openapi.paths.paths {
            let tag = tag_for_path(path);
            set_operation(&mut item.get, tag, operation_summary("GET", path));
            set_operation(&mut item.post, tag, operation_summary("POST", path));
            set_operation(&mut item.put, tag, operation_summary("PUT", path));
            set_operation(&mut item.delete, tag, operation_summary("DELETE", path));
        }
    }
}

fn set_operation(operation: &mut Option<Operation>, tag: &str, summary: &str) {
    if let Some(operation) = operation {
        operation.tags = Some(vec![tag.to_owned()]);
        operation.summary = Some(summary.to_owned());
    }
}

fn tag_for_path(path: &str) -> &'static str {
    match path {
        "/api/incomes" | "/api/incomes/{id}" => "収入",
        "/api/income-categories" | "/api/income-categories/{id}" => "収入カテゴリ",
        "/api/expenses" | "/api/expenses/{id}" => "支出",
        "/api/expense-categories" | "/api/expense-categories/{id}" => "支出カテゴリ",
        "/api/export/expenses" | "/api/export/incomes" => "エクスポート",
        "/api/import/expenses" | "/api/import/incomes" => "インポート",
        "/api/payment-methods" | "/api/payment-methods/{id}" => "支払方法",
        "/api/recurring-expenses" | "/api/recurring-expenses/{id}" => "定期支出",
        "/api/recurring-incomes" | "/api/recurring-incomes/{id}" => "定期収入",
        "/api/webhook-urls" | "/api/webhook-urls/{id}" => "Webhook",
        "/api/budgets" | "/api/budgets/{id}" => "予算",
        _ => "システム",
    }
}

fn operation_summary(method: &str, path: &str) -> &'static str {
    match (method, path.ends_with("{id}")) {
        ("GET", false) if path == "/health" => "稼働状態を確認",
        ("GET", false) if path.starts_with("/api/export/") => "CSVをエクスポート",
        ("GET", false) => "一覧を取得",
        ("GET", true) => "詳細を取得",
        ("POST", _) if path.starts_with("/api/import/") => "CSVをインポート",
        ("POST", _) => "新規登録",
        ("PUT", _) => "更新",
        ("DELETE", _) => "削除",
        _ => "API操作",
    }
}

pub fn create() -> Router {
    let config = || {
        json!({
            "disableSearch": false,
            "expandResponses": "200,201",
            "hideDownloadButton": false,
            "hideHostname": true,
            "nativeScrollbars": true,
            "pathInMiddlePanel": true,
            "requiredPropsFirst": true,
            "sortPropsAlphabetically": true,
            "theme": {
                "colors": {
                    "primary": { "main": "#0f766e" },
                    "success": { "main": "#15803d" }
                },
                "sidebar": {
                    "backgroundColor": "#f8fafc",
                    "textColor": "#334155",
                    "activeTextColor": "#0f766e",
                    "width": "300px"
                },
                "typography": {
                    "fontFamily": "'Noto Sans JP', sans-serif",
                    "fontSize": "15px",
                    "headings": {
                        "fontFamily": "'Noto Sans JP', sans-serif",
                        "fontWeight": "700"
                    }
                }
            }
        })
    };

    Router::new().merge(Redoc::with_url_and_config(
        "/docs",
        ApiDoc::openapi(),
        config,
    ))
}
