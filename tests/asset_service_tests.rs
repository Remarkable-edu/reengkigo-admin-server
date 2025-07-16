mod common;

use server_test::services::asset::AssetService;
use common::{setup_test_app, cleanup_test_db, sample_asset_request};

#[tokio::test]
async fn test_create_asset() {
    let state = setup_test_app().await;
    let request = sample_asset_request();

    let result = AssetService::create_asset(&state.db, request).await;
    
    assert!(result.is_ok());
    let asset = result.unwrap();
    assert!(!asset.id.is_empty());
    assert_eq!(asset.curriculum.len(), 1);
    assert_eq!(asset.curriculum[0].books.len(), 1);
    assert_eq!(asset.curriculum[0].books[0].book_id, "J1R".to_string());
    assert_eq!(asset.curriculum[0].books[0].month, "January");

    cleanup_test_db(&state).await;
}

#[tokio::test]
async fn test_create_asset_with_multiple_books() {
    let state = setup_test_app().await;
    
    let request = server_test::dto::asset::CreateAssetRequest {
        curriculum: vec![
            server_test::dto::asset::CreateCurriculumRequest {
                id: "curriculum_1".to_string(),
                books: vec![
                    server_test::dto::asset::CreateBookRequest {
                        book_id: "J1R".to_string(),
                        month: "January".to_string(),
                        cover_img: "test/cover1.jpg".to_string(),
                        video_content: vec![],
                    },
                    server_test::dto::asset::CreateBookRequest {
                        book_id: "F2R".to_string(),
                        month: "February".to_string(),
                        cover_img: "test/cover2.jpg".to_string(),
                        video_content: vec![
                            server_test::dto::asset::CreateVideoContentRequest {
                                video_img: "test/video2.jpg".to_string(),
                                youtube_url: "https://youtube.com/watch?v=test456".to_string(),
                            }
                        ],
                    }
                ],
            }
        ],
    };

    let result = AssetService::create_asset(&state.db, request).await;
    
    assert!(result.is_ok());
    let asset = result.unwrap();
    assert_eq!(asset.curriculum.len(), 1);
    assert_eq!(asset.curriculum[0].books.len(), 2);
    assert_eq!(asset.curriculum[0].books[0].book_id, "J1R".to_string());
    assert_eq!(asset.curriculum[0].books[1].book_id, "F2R".to_string());
    assert_eq!(asset.curriculum[0].books[0].video_content.len(), 0);
    assert_eq!(asset.curriculum[0].books[1].video_content.len(), 1);

    cleanup_test_db(&state).await;
}

#[tokio::test]
async fn test_create_asset_with_video_content() {
    let state = setup_test_app().await;
    
    let request = server_test::dto::asset::CreateAssetRequest {
        curriculum: vec![
            server_test::dto::asset::CreateCurriculumRequest {
                id: "curriculum_1".to_string(),
                books: vec![
                    server_test::dto::asset::CreateBookRequest {
                        book_id: "M3R".to_string(),
                        month: "March".to_string(),
                        cover_img: "test/march_cover.jpg".to_string(),
                        video_content: vec![
                            server_test::dto::asset::CreateVideoContentRequest {
                                video_img: "test/video1.jpg".to_string(),
                                youtube_url: "https://youtube.com/watch?v=lesson1".to_string(),
                            },
                            server_test::dto::asset::CreateVideoContentRequest {
                                video_img: "test/video2.jpg".to_string(),
                                youtube_url: "https://youtube.com/watch?v=lesson2".to_string(),
                            }
                        ],
                    }
                ],
            }
        ],
    };

    let result = AssetService::create_asset(&state.db, request).await;
    
    assert!(result.is_ok());
    let asset = result.unwrap();
    assert_eq!(asset.curriculum.len(), 1);
    assert_eq!(asset.curriculum[0].books.len(), 1);
    assert_eq!(asset.curriculum[0].books[0].book_id, "M3R".to_string());
    assert_eq!(asset.curriculum[0].books[0].month, "March");
    assert_eq!(asset.curriculum[0].books[0].video_content.len(), 2);
    assert_eq!(asset.curriculum[0].books[0].video_content[0].youtube_url, "https://youtube.com/watch?v=lesson1");
    assert_eq!(asset.curriculum[0].books[0].video_content[1].youtube_url, "https://youtube.com/watch?v=lesson2");

    cleanup_test_db(&state).await;
}

#[tokio::test]
async fn test_create_asset_with_empty_video_content() {
    let state = setup_test_app().await;
    
    let request = server_test::dto::asset::CreateAssetRequest {
        curriculum: vec![
            server_test::dto::asset::CreateCurriculumRequest {
                id: "curriculum_1".to_string(),
                books: vec![
                    server_test::dto::asset::CreateBookRequest {
                        book_id: "A4R".to_string(),
                month: "April".to_string(),
                cover_img: "test/april_cover.jpg".to_string(),
                video_content: vec![],
                    }
                ],
            }
        ],
    };

    let result = AssetService::create_asset(&state.db, request).await;
    
    assert!(result.is_ok());
    let asset = result.unwrap();
    assert_eq!(asset.books.len(), 1);
    assert_eq!(asset.books[0].book_id, Some("A4R".to_string()));
    assert_eq!(asset.books[0].month, "April");
    assert_eq!(asset.books[0].video_content.len(), 0);

    cleanup_test_db(&state).await;
}

#[tokio::test]
async fn test_create_asset_without_book_id() {
    let state = setup_test_app().await;
    
    let request = server_test::dto::asset::CreateAssetRequest {
        books: vec![
            server_test::dto::asset::CreateBookRequest {
                book_id: None,
                month: "May".to_string(),
                cover_img: "test/may_cover.jpg".to_string(),
                video_content: vec![
                    server_test::dto::asset::CreateVideoContentRequest {
                        video_img: "test/may_video.jpg".to_string(),
                        youtube_url: "https://youtube.com/watch?v=may_lesson".to_string(),
                    }
                ],
            }
        ],
    };

    let result = AssetService::create_asset(&state.db, request).await;
    
    assert!(result.is_ok());
    let asset = result.unwrap();
    assert_eq!(asset.books.len(), 1);
    assert_eq!(asset.books[0].book_id, None);
    assert_eq!(asset.books[0].month, "May");
    assert_eq!(asset.books[0].video_content.len(), 1);

    cleanup_test_db(&state).await;
}