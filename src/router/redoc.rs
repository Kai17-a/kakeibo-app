use axum::Router;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};

use crate::{
    handler::{
        expense_categories, expenses, income_categories, incomes, payment_methods,
        recurring_expenses,
    },
    model::expense_categories::{
        ExpenseCategory, ExpenseCategoryListResponse, ExpenseCategoryPagination,
        ExpenseCategorySortBy, ExpenseCategorySortOrder, ExpenseCategoryUpsertRequest,
    },
    model::expenses::{Expense, ExpenseUpsertRequest},
    model::income_categories::{
        IncomeCategory, IncomeCategoryListResponse, IncomeCategoryPagination, IncomeCategorySortBy,
        IncomeCategorySortOrder, IncomeCategoryUpsertRequest,
    },
    model::incomes::{
        Income, IncomeListResponse, IncomeSortBy, IncomeUpsertRequest, Pagination, SortOrder,
    },
    model::payment_methods::{
        PaymentMethod, PaymentMethodListResponse, PaymentMethodPagination, PaymentMethodSortBy,
        PaymentMethodSortOrder, PaymentMethodUpsertRequest,
    },
    model::recurring_expenses::{RecurringExpense, RecurringExpenseUpsertRequest},
};

#[derive(OpenApi)]
#[openapi(
    paths(
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
        income_categories::list,
        income_categories::get,
        income_categories::create,
        income_categories::update,
        income_categories::delete,
        incomes::list,
        incomes::get,
        incomes::create,
        incomes::update,
        incomes::delete
    ),
    components(schemas(
        ExpenseCategory,
        ExpenseCategoryUpsertRequest,
        ExpenseCategoryListResponse,
        ExpenseCategoryPagination,
        ExpenseCategorySortBy,
        ExpenseCategorySortOrder,
        Expense,
        ExpenseUpsertRequest,
        PaymentMethod,
        PaymentMethodUpsertRequest,
        PaymentMethodListResponse,
        PaymentMethodPagination,
        PaymentMethodSortBy,
        PaymentMethodSortOrder,
        RecurringExpense,
        RecurringExpenseUpsertRequest,
        IncomeCategory,
        IncomeCategoryUpsertRequest,
        IncomeCategoryListResponse,
        IncomeCategoryPagination,
        IncomeCategorySortBy,
        IncomeCategorySortOrder,
        Income,
        IncomeUpsertRequest,
        IncomeListResponse,
        Pagination,
        IncomeSortBy,
        SortOrder
    )),
    tags(
        (name = "expense categories", description = "Expense category CRUD API"),
        (name = "expenses", description = "Expense CRUD API"),
        (name = "payment methods", description = "Payment method CRUD API"),
        (name = "recurring expenses", description = "Recurring expense CRUD API"),
        (name = "income categories", description = "Income category CRUD API"),
        (name = "incomes", description = "Income CRUD API")
    )
)]
struct ApiDoc;

pub fn create() -> Router {
    Router::new().merge(Redoc::with_url("/docs", ApiDoc::openapi()))
}
