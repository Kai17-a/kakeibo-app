use crate::{
    database::models::expenses::ExpenseRow, model::expenses::ExpenseUpsertRequest,
    utils::error::AppResult,
};
use sqlx::SqlitePool;

#[derive(Debug)]
pub struct BudgetCrossing {
    pub category_id: String,
    pub category_name: String,
    pub budget_amount: String,
    pub actual_amount: i64,
    pub target_month: String,
}

pub struct ExpenseWriteResult {
    pub expense: ExpenseRow,
    pub budget_crossing: Option<BudgetCrossing>,
}
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
        Ok(sqlx::query(include_str!(
            "../../queries/expenses/insert_recurring_for_month.sql"
        ))
        .bind(month)
        .execute(&self.pool)
        .await?
        .rows_affected())
    }
    pub async fn find_by_id(&self, id: &str) -> AppResult<Option<ExpenseRow>> {
        sqlx::query_as(include_str!("../../queries/expenses/find_by_id.sql"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn insert(&self, v: &ExpenseUpsertRequest) -> AppResult<ExpenseRow> {
        bind(
            sqlx::query_as(include_str!("../../queries/expenses/insert.sql")),
            v,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }
    pub async fn insert_with_budget_check(
        &self,
        v: &ExpenseUpsertRequest,
    ) -> AppResult<ExpenseWriteResult> {
        let mut tx = self.pool.begin().await?;
        let expense = bind(
            sqlx::query_as(include_str!("../../queries/expenses/insert.sql")),
            v,
        )
        .fetch_one(&mut *tx)
        .await?;
        let month = v.transaction_date.get(..7).unwrap_or_default();
        let crossing = match budget_state(&mut tx, &v.category_id, month).await {
            Ok(Some((budget_amount, category_name, after_total))) => {
                let budget = budget_amount.parse::<i64>().unwrap_or(i64::MAX);
                let amount = v
                    .amount
                    .parse::<i64>()
                    .expect("amount validated as i64-range integer");
                (after_total - amount <= budget && after_total > budget).then(|| BudgetCrossing {
                    category_id: v.category_id.clone(),
                    category_name,
                    budget_amount,
                    actual_amount: after_total,
                    target_month: month.to_owned(),
                })
            }
            Ok(None) => None,
            Err(error) => {
                tracing::warn!(%error, "Failed to evaluate budget after expense insert");
                None
            }
        };
        tx.commit().await?;
        Ok(ExpenseWriteResult {
            expense,
            budget_crossing: crossing,
        })
    }
    pub async fn update(
        &self,
        id: &str,
        v: &ExpenseUpsertRequest,
    ) -> AppResult<Option<ExpenseRow>> {
        bind(
            sqlx::query_as(include_str!("../../queries/expenses/update.sql")),
            v,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }
    pub async fn update_with_budget_check(
        &self,
        id: &str,
        v: &ExpenseUpsertRequest,
    ) -> AppResult<Option<ExpenseWriteResult>> {
        let mut tx = self.pool.begin().await?;
        let old: Option<ExpenseRow> =
            sqlx::query_as(include_str!("../../queries/expenses/find_by_id.sql"))
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?;
        let Some(old) = old else {
            tx.rollback().await?;
            return Ok(None);
        };
        let expense = bind(
            sqlx::query_as(include_str!("../../queries/expenses/update.sql")),
            v,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
        let month = v.transaction_date.get(..7).unwrap_or_default();
        let crossing = match budget_state(&mut tx, &v.category_id, month).await {
            Ok(Some((budget_amount, category_name, after_total))) => {
                let budget = budget_amount.parse::<i64>().unwrap_or(i64::MAX);
                let new_amount = v
                    .amount
                    .parse::<i64>()
                    .expect("amount validated as i64-range integer");
                let same_bucket = old.category_id == v.category_id
                    && old.transaction_date.get(..7).unwrap_or_default() == month;
                let before_total = if same_bucket {
                    after_total - new_amount
                        + old
                            .amount
                            .parse::<i64>()
                            .expect("amount validated as i64-range integer")
                } else {
                    after_total - new_amount
                };
                (before_total <= budget && after_total > budget).then(|| BudgetCrossing {
                    category_id: v.category_id.clone(),
                    category_name,
                    budget_amount,
                    actual_amount: after_total,
                    target_month: month.to_owned(),
                })
            }
            Ok(None) => None,
            Err(error) => {
                tracing::warn!(%error, "Failed to evaluate budget after expense update");
                None
            }
        };
        tx.commit().await?;
        Ok(Some(ExpenseWriteResult {
            expense,
            budget_crossing: crossing,
        }))
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

async fn budget_state(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    category_id: &str,
    month: &str,
) -> Result<Option<(String, String, i64)>, sqlx::Error> {
    sqlx::query_as(include_str!(
        "../../queries/expenses/category_month_budget.sql"
    ))
    .bind(category_id)
    .bind(month)
    .fetch_optional(&mut **tx)
    .await
}
fn bind<'q>(
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
