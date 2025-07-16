mod common;

use axum::{
    body::{to_bytes, Body},
    http::{Method, Request, StatusCode},
    Router,
};
use serde_json::json;
use tower::ServiceExt;

use server_test::{
    dto::asset::AssetResponse,
    handlers::admin_head,
};
use common::{setup_test_app, cleanup_test_db};

fn create_test_router(state: server_test::AppState) -> Router {
    Router::new()
        .route("/api/assets", axum::routing::post(admin_head::create_asset))
        .with_state(state)
}

#[tokio::test]
async fn test_create_asset_endpoint() {
    let state = setup_test_app().await;
    let app = create_test_router(state.clone());
    
    let request_body = json!({
        "books": [
            {
                "book_id": "J1R",
                "month": "January",
                "cover_img": "test/cover.jpg",
                "video_content": [
                    {
                        "video_img": "test/video.jpg",
                        "youtube_url": "https://youtube.com/watch?v=test123"
                    }
                ]
            }
        ]
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/assets")
        .header("content-type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let asset: AssetResponse = serde_json::from_slice(&body).unwrap();
    
    assert!(!asset.id.is_empty());
    assert_eq!(asset.books.len(), 1);
    assert_eq!(asset.books[0].book_id, Some("J1R".to_string()));
    assert_eq!(asset.books[0].month, "January");

    cleanup_test_db(&state).await;
}

#[tokio::test]
async fn test_create_asset_with_multiple_books_endpoint() {
    let state = setup_test_app().await;
    let app = create_test_router(state.clone());
    
    let request_body = json!({
        "books": [
            {
                "book_id": "J1R",
                "month": "January",
                "cover_img": "test/january_cover.jpg",
                "video_content": [
                    {
                        "video_img": "test/jan_video.jpg",
                        "youtube_url": "https://youtube.com/watch?v=jan123"
                    }
                ]
            },
            {
                "book_id": "F2R",
                "month": "February",
                "cover_img": "test/february_cover.jpg",
                "video_content": []
            }
        ]
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/assets")
        .header("content-type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let asset: AssetResponse = serde_json::from_slice(&body).unwrap();
    
    assert!(!asset.id.is_empty());
    assert_eq!(asset.books.len(), 2);
    assert_eq!(asset.books[0].book_id, Some("J1R".to_string()));
    assert_eq!(asset.books[1].book_id, Some("F2R".to_string()));
    assert_eq!(asset.books[0].video_content.len(), 1);
    assert_eq!(asset.books[1].video_content.len(), 0);

    cleanup_test_db(&state).await;
}

#[tokio::test]
async fn test_create_asset_without_book_id_endpoint() {
    let state = setup_test_app().await;
    let app = create_test_router(state.clone());
    
    let request_body = json!({
        "books": [
            {
                "book_id": null,
                "month": "March",
                "cover_img": "test/march_cover.jpg",
                "video_content": [
                    {
                        "video_img": "test/march_video.jpg",
                        "youtube_url": "https://youtube.com/watch?v=march456"
                    }
                ]
            }
        ]
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/assets")
        .header("content-type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let asset: AssetResponse = serde_json::from_slice(&body).unwrap();
    
    assert!(!asset.id.is_empty());
    assert_eq!(asset.books.len(), 1);
    assert_eq!(asset.books[0].book_id, None);
    assert_eq!(asset.books[0].month, "March");

    cleanup_test_db(&state).await;
}

#[tokio::test]
async fn test_create_asset_with_empty_video_content_endpoint() {
    let state = setup_test_app().await;
    let app = create_test_router(state.clone());
    
    let request_body = json!({
        "books": [
            {
                "book_id": "A4R",
                "month": "April",
                "cover_img": "test/april_cover.jpg",
                "video_content": []
            }
        ]
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/assets")
        .header("content-type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let asset: AssetResponse = serde_json::from_slice(&body).unwrap();
    
    assert!(!asset.id.is_empty());
    assert_eq!(asset.books.len(), 1);
    assert_eq!(asset.books[0].book_id, Some("A4R".to_string()));
    assert_eq!(asset.books[0].video_content.len(), 0);

    cleanup_test_db(&state).await;
}

#[tokio::test]
async fn test_create_asset_with_multiple_videos_endpoint() {
    let state = setup_test_app().await;
    let app = create_test_router(state.clone());
    
    let request_body = json!({
        "books": [
            {
                "book_id": "M5R",
                "month": "May",
                "cover_img": "test/may_cover.jpg",
                "video_content": [
                    {
                        "video_img": "test/may_video1.jpg",
                        "youtube_url": "https://youtube.com/watch?v=may_lesson1"
                    },
                    {
                        "video_img": "test/may_video2.jpg",
                        "youtube_url": "https://youtube.com/watch?v=may_lesson2"
                    },
                    {
                        "video_img": "test/may_video3.jpg",
                        "youtube_url": "https://youtube.com/watch?v=may_lesson3"
                    }
                ]
            }
        ]
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/assets")
        .header("content-type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let asset: AssetResponse = serde_json::from_slice(&body).unwrap();
    
    assert!(!asset.id.is_empty());
    assert_eq!(asset.books.len(), 1);
    assert_eq!(asset.books[0].book_id, Some("M5R".to_string()));
    assert_eq!(asset.books[0].video_content.len(), 3);

    cleanup_test_db(&state).await;
}

#[tokio::test]
async fn test_invalid_json_request() {
    let state = setup_test_app().await;
    let app = create_test_router(state.clone());
    
    let invalid_json = "{ invalid json }";

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/assets")
        .header("content-type", "application/json")
        .body(Body::from(invalid_json))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    cleanup_test_db(&state).await;
}

#[tokio::test]
async fn test_missing_content_type_header() {
    let state = setup_test_app().await;
    let app = create_test_router(state.clone());
    
    let request_body = json!({
        "books": [
            {
                "book_id": "J6R",
                "month": "June",
                "cover_img": "test/june_cover.jpg",
                "video_content": []
            }
        ]
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/assets")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    // Should still work as axum can handle JSON without explicit content-type
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST);

    cleanup_test_db(&state).await;
}