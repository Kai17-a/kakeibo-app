use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::router::redoc;
use tower::ServiceExt;

#[tokio::test]
async fn serves_api_documentation() {
    let response = redoc::create()
        .oneshot(Request::get("/docs").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()["content-type"],
        "text/html; charset=utf-8"
    );
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("redoc"));
    assert!(html.contains("/api/expense-categories"));
    assert!(html.contains("/api/payment-methods"));
    assert!(html.contains("/api/recurring-expenses"));
    assert!(html.contains("/api/expenses"));
    assert!(html.contains("/api/income-categories"));
    assert!(html.contains("/health"));
    assert!(html.contains("Kakeibo API"));
    assert!(html.contains("収入カテゴリ"));
    assert!(html.contains("支出カテゴリ"));
    assert!(html.contains(r##""primary":{"main":"#0f766e"}"##));
    assert!(html.contains(r#""hideDownloadButton":false"#));
    assert!(html.contains(r#""name":"page""#));
    assert!(html.contains(r#""default":1"#));
    assert!(html.contains(r#""name":"per_page""#));
    assert!(html.contains(r#""default":50"#));
    // ダウンロードボタンはHTMLへインライン埋め込みされたOpenAPI定義から生成される
    assert!(html.contains(r#""openapi":"3.1.0""#));
}
