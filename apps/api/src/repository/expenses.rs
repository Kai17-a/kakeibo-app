use crate::{
    database::models::{expenses::ExpenseRow, recurring_expenses::RecurringExpenseRow},
    model::expenses::ExpenseUpsertRequest,
    utils::error::AppResult,
};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct ExpenseRepository {
    pool: SqlitePool,
}
impl ExpenseRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    pub async fn find_all(&self) -> AppResult<Vec<ExpenseRow>> {
        sqlx::query_as(include_str!("../../queries/expenses/find_all.sql"))
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn current_month(&self) -> AppResult<String> {
        let (month,): (String,) =
            sqlx::query_as(include_str!("../../queries/expenses/current_month.sql"))
                .fetch_one(&self.pool)
                .await?;
        Ok(month)
    }
    pub async fn insert_recurring_for_month(&self, month: &str) -> AppResult<u64> {
        let recurring = self.find_recurring_for_month(month).await?;
        let mut inserted = 0;
        for item in recurring {
            if self.insert_recurring(month, &item, None).await? {
                inserted += 1;
            }
        }
        Ok(inserted)
    }
    pub async fn find_recurring_for_month(
        &self,
        month: &str,
    ) -> AppResult<Vec<RecurringExpenseRow>> {
        sqlx::query_as(include_str!(
            "../../queries/expenses/find_recurring_for_month.sql"
        ))
        .bind(month)
        .bind(month)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }
    pub async fn find_recurring_expense_months(&self, id: &str) -> AppResult<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(include_str!(
            "../../queries/expenses/find_recurring_expense_months.sql"
        ))
        .bind(id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|(month,)| month).collect())
    }
    pub async fn insert_recurring(
        &self,
        month: &str,
        recurring: &RecurringExpenseRow,
        conversion: Option<(&str, &str, &str, &str)>,
    ) -> AppResult<bool> {
        if recurring.currency_code.as_deref() == Some("USD") && conversion.is_none() {
            return Ok(false);
        }
        let mut query = sqlx::query(
            "INSERT INTO expenses (transaction_date, amount, category_id, payment_method_id, recurring_expense_id, description, foreign_amount, currency_code, exchange_rate, exchange_rate_date) SELECT date(? || '-01', '+' || (min(?, cast(strftime('%d', date(? || '-01', '+1 month', '-1 day')) AS INTEGER)) - 1) || ' days'), ?, ?, ?, ?, ?, ?, ?, ?, ? WHERE NOT EXISTS (SELECT 1 FROM expenses WHERE recurring_expense_id = ? AND strftime('%Y-%m', transaction_date) = ?)",
        );
        let (amount, foreign_amount, currency_code, rate, rate_date) = match conversion {
            Some((amount, foreign_amount, rate, rate_date)) => (
                amount,
                Some(foreign_amount),
                Some("USD"),
                Some(rate),
                Some(rate_date),
            ),
            None => (
                recurring.amount.as_str(),
                recurring.foreign_amount.as_deref(),
                recurring.currency_code.as_deref(),
                None,
                None,
            ),
        };
        query = query
            .bind(month)
            .bind(recurring.payment_day)
            .bind(month)
            .bind(amount)
            .bind(&recurring.category_id)
            .bind(&recurring.payment_method_id)
            .bind(&recurring.id)
            .bind(&recurring.description)
            .bind(foreign_amount)
            .bind(currency_code)
            .bind(rate)
            .bind(rate_date)
            .bind(&recurring.id)
            .bind(month);
        Ok(query.execute(&self.pool).await?.rows_affected() > 0)
    }
    pub async fn find_by_id(&self, id: &str) -> AppResult<Option<ExpenseRow>> {
        sqlx::query_as(include_str!("../../queries/expenses/find_by_id.sql"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn insert(&self, v: &ExpenseUpsertRequest) -> AppResult<ExpenseRow> {
        bind_insert(
            sqlx::query_as(include_str!("../../queries/expenses/insert.sql")),
            v,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }
    pub async fn update(
        &self,
        id: &str,
        v: &ExpenseUpsertRequest,
    ) -> AppResult<Option<ExpenseRow>> {
        bind_update(
            sqlx::query_as(include_str!("../../queries/expenses/update.sql")),
            v,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }
    pub async fn delete(&self, id: &str) -> AppResult<bool> {
        Ok(
            sqlx::query(include_str!("../../queries/expenses/delete_by_id.sql"))
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected()
                > 0,
        )
    }
}

fn bind_insert<'q>(
    q: sqlx::query::QueryAs<'q, sqlx::Sqlite, ExpenseRow, sqlx::sqlite::SqliteArguments>,
    v: &'q ExpenseUpsertRequest,
) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, ExpenseRow, sqlx::sqlite::SqliteArguments> {
    q.bind(&v.transaction_date)
        .bind(&v.amount)
        .bind(&v.category_id)
        .bind(&v.payment_method_id)
        .bind(&v.recurring_expense_id)
        .bind(&v.description)
        .bind(&v.foreign_amount)
        .bind(&v.currency_code)
        .bind(&v.exchange_rate)
        .bind(&v.exchange_rate_date)
}

fn bind_update<'q>(
    q: sqlx::query::QueryAs<'q, sqlx::Sqlite, ExpenseRow, sqlx::sqlite::SqliteArguments>,
    v: &'q ExpenseUpsertRequest,
) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, ExpenseRow, sqlx::sqlite::SqliteArguments> {
    q.bind(&v.transaction_date)
        .bind(&v.amount)
        .bind(&v.category_id)
        .bind(&v.payment_method_id)
        .bind(&v.recurring_expense_id)
        .bind(&v.description)
}
