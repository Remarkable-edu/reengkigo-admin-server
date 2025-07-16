use server_test::{
    config::AppConfig,
    services::database::Database,
    utils::ObservabilityManager,
    AppState,
};
use std::sync::Arc;

pub async fn setup_test_app() -> AppState {
    dotenv::dotenv().ok();
    
    let config = Arc::new(AppConfig::default());
    
    let test_db_name = format!("test_db_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
    
    let database = Database::new(
        &config.database.url,
        &test_db_name
    ).await.expect("Failed to connect to test database");

    let observability = Arc::new(
        ObservabilityManager::new(config.clone())
            .await
            .expect("Failed to create observability manager")
    );

    AppState {
        db: database,
        config,
        observability,
    }
}

pub async fn cleanup_test_db(state: &AppState) {
    state.db.database.drop(None).await.expect("Failed to drop test database");
}

pub fn sample_asset_request() -> server_test::dto::asset::CreateAssetRequest {
    server_test::dto::asset::CreateAssetRequest {
        curriculum: vec![
            server_test::dto::asset::CreateCurriculumRequest {
                id: "curriculum_1".to_string(),
                books: vec![
                    server_test::dto::asset::CreateBookRequest {
                        book_id: "J1R".to_string(),
                        month: "January".to_string(),
                        cover_img: "test/cover.jpg".to_string(),
                        video_content: vec![
                            server_test::dto::asset::CreateVideoContentRequest {
                                video_img: "test/video.jpg".to_string(),
                                youtube_url: "https://youtube.com/watch?v=test123".to_string(),
                            }
                        ],
                    }
                ],
            }
        ],
    }
}

pub fn sample_asset_request_with_multiple_books() -> server_test::dto::asset::CreateAssetRequest {
    server_test::dto::asset::CreateAssetRequest {
        curriculum: vec![
            server_test::dto::asset::CreateCurriculumRequest {
                id: "curriculum_1".to_string(),
                books: vec![
                    server_test::dto::asset::CreateBookRequest {
                        book_id: "J1R".to_string(),
                        month: "January".to_string(),
                        cover_img: "test/january_cover.jpg".to_string(),
                        video_content: vec![
                            server_test::dto::asset::CreateVideoContentRequest {
                                video_img: "test/january_video.jpg".to_string(),
                                youtube_url: "https://youtube.com/watch?v=jan123".to_string(),
                            }
                        ],
                    },
                    server_test::dto::asset::CreateBookRequest {
                        book_id: "F2R".to_string(),
                        month: "February".to_string(),
                        cover_img: "test/february_cover.jpg".to_string(),
                        video_content: vec![],
                    }
                ],
            }
        ],
    }
}

pub fn sample_asset_request_without_book_id() -> server_test::dto::asset::CreateAssetRequest {
    server_test::dto::asset::CreateAssetRequest {
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
                                video_img: "test/march_video.jpg".to_string(),
                                youtube_url: "https://youtube.com/watch?v=march456".to_string(),
                            }
                        ],
                    }
                ],
            }
        ],
    }
}