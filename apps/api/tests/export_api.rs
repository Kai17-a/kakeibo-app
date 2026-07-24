use axum::{
    body::{Body, to_bytes},
    http::{HeaderMap, Request, StatusCode},
};
use kakeibo_app::router::export;
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;
async fn call(app: &axum::Router, u: &str) -> (StatusCode, HeaderMap, String) {
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(u)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let s = r.status();
    let h = r.headers().clone();
    let bytes = to_bytes(r.into_body(), usize::MAX).await.unwrap();
    (s, h, String::from_utf8(bytes.to_vec()).unwrap())
}
async fn pool(schema: &'static str) -> sqlx::SqlitePool {
    let p = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query(schema).execute(&p).await.unwrap();
    p
}
#[tokio::test]
async fn export_expenses_csv() {
    let p = pool("CREATE TABLE expense_categories(id TEXT PRIMARY KEY,name TEXT NOT NULL);CREATE TABLE payment_methods(id TEXT PRIMARY KEY,name TEXT NOT NULL);CREATE TABLE recurring_expenses(id TEXT PRIMARY KEY);CREATE TABLE expenses(id TEXT PRIMARY KEY,created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,transaction_date TEXT NOT NULL,amount TEXT NOT NULL,category_id TEXT NOT NULL REFERENCES expense_categories(id),payment_method_id TEXT NOT NULL REFERENCES payment_methods(id),recurring_expense_id TEXT REFERENCES recurring_expenses(id),description TEXT);INSERT INTO expense_categories VALUES('ec','食費');INSERT INTO payment_methods VALUES('pm','現金');INSERT INTO expenses(id,transaction_date,amount,category_id,payment_method_id,description) VALUES('e-1','2026-07-22','1200','ec','pm','ランチ, 友人と'),('e-2','2026-07-21','500','ec','pm',NULL)").await;
    let app = export::create(p);
    let (s, h, b) = call(&app, "/api/export/expenses").await;
    assert_eq!(s, StatusCode::OK, "{b}");
    assert_eq!(h["content-type"], "text/csv; charset=utf-8");
    assert_eq!(
        h["content-disposition"],
        "attachment; filename=\"expenses.csv\""
    );
    assert_eq!(
        b,
        "\u{feff}日付,金額,カテゴリ,支払方法,メモ\n2026-07-21,500,食費,現金,\n2026-07-22,1200,食費,現金,\"ランチ, 友人と\"\n"
    );
}
#[tokio::test]
async fn export_incomes_csv() {
    let p = pool("CREATE TABLE income_categories(id TEXT PRIMARY KEY,name TEXT NOT NULL);CREATE TABLE incomes(id TEXT PRIMARY KEY,created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,category_id TEXT NOT NULL REFERENCES income_categories(id),transaction_date TEXT NOT NULL,amount TEXT NOT NULL,description TEXT);INSERT INTO income_categories VALUES('ic','給与');INSERT INTO incomes(id,category_id,transaction_date,amount,description) VALUES('i-1','ic','2026-07-25','300000','7月分')").await;
    let app = export::create(p);
    let (s, h, b) = call(&app, "/api/export/incomes").await;
    assert_eq!(s, StatusCode::OK, "{b}");
    assert_eq!(h["content-type"], "text/csv; charset=utf-8");
    assert_eq!(
        h["content-disposition"],
        "attachment; filename=\"incomes.csv\""
    );
    assert_eq!(
        b,
        "\u{feff}日付,金額,カテゴリ,メモ\n2026-07-25,300000,給与,7月分\n"
    );
}
