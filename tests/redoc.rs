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
    assert!(html.contains(r#""name":"page""#));
    assert!(html.contains(r#""default":1"#));
    assert!(html.contains(r#""name":"per_page""#));
    assert!(html.contains(r#""default":50"#));
}
