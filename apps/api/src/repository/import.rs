use crate::{
    model::{expenses::ExpenseUpsertRequest, incomes::IncomeUpsertRequest},
    utils::error::AppResult,
};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
#[derive(Clone)]
pub struct ImportRepository {
    pool: SqlitePool,
}
impl ImportRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    pub async fn begin(&self) -> AppResult<Transaction<'static, Sqlite>> {
        Ok(self.pool.begin().await?)
    }
    pub async fn find_expense_category_id(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        name: &str,
    ) -> AppResult<Option<String>> {
        find_id_by_name(
            tx,
            include_str!("../../queries/import/find_expense_category_id_by_name.sql"),
            name,
        )
        .await
    }
    pub async fn find_income_category_id(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        name: &str,
    ) -> AppResult<Option<String>> {
        find_id_by_name(
            tx,
            include_str!("../../queries/import/find_income_category_id_by_name.sql"),
            name,
        )
        .await
    }
    pub async fn find_payment_method_id(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        name: &str,
    ) -> AppResult<Option<String>> {
        find_id_by_name(
            tx,
            include_str!("../../queries/import/find_payment_method_id_by_name.sql"),
            name,
        )
        .await
    }
    pub async fn create_expense_category(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        name: &str,
    ) -> AppResult<String> {
        insert_named(
            tx,
            include_str!("../../queries/expense_categories/insert.sql"),
            name,
        )
        .await
    }
    pub async fn create_income_category(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        name: &str,
    ) -> AppResult<String> {
        insert_named(
            tx,
            include_str!("../../queries/income_categories/insert.sql"),
            name,
        )
        .await
    }
    pub async fn create_payment_method(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        name: &str,
    ) -> AppResult<String> {
        Ok(sqlx::query(
            "INSERT INTO payment_methods (name, description, initial_balance) \
             VALUES (?1, NULL, NULL) RETURNING id",
        )
        .bind(name)
        .fetch_one(&mut **tx)
        .await?
        .get("id"))
    }
    pub async fn insert_expense(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        v: &ExpenseUpsertRequest,
    ) -> AppResult<()> {
        sqlx::query(include_str!("../../queries/expenses/insert.sql"))
            .bind(&v.transaction_date)
            .bind(&v.amount)
            .bind(&v.category_id)
            .bind(&v.payment_method_id)
            .bind(&v.recurring_expense_id)
            .bind(&v.description)
            .fetch_one(&mut **tx)
            .await?;
        Ok(())
    }
    pub async fn insert_income(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        v: &IncomeUpsertRequest,
    ) -> AppResult<()> {
        sqlx::query(include_str!("../../queries/incomes/insert.sql"))
            .bind(&v.category_id)
            .bind(&v.transaction_date)
            .bind(&v.amount)
            .bind(&v.payment_method_id)
            .bind(&v.recurring_income_id)
            .bind(&v.description)
            .fetch_one(&mut **tx)
            .await?;
        Ok(())
    }
}
async fn find_id_by_name(
    tx: &mut Transaction<'_, Sqlite>,
    query: &'static str,
    name: &str,
) -> AppResult<Option<String>> {
    Ok(sqlx::query(query)
        .bind(name)
        .fetch_optional(&mut **tx)
        .await?
        .map(|row| row.get("id")))
}
async fn insert_named(
    tx: &mut Transaction<'_, Sqlite>,
    query: &'static str,
    name: &str,
) -> AppResult<String> {
    Ok(sqlx::query(query)
        .bind(name)
        .bind(None::<String>)
        .fetch_one(&mut **tx)
        .await?
        .get("id"))
}
