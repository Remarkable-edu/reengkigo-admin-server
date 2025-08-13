use axum::{extract::Query, response::Json};
use serde::Deserialize;
use crate::{
    dto::folder_category::{FolderCategoryListResponse, FolderCategoryResponse, CourseGroupResponse},
    models::folder_category::FolderCategory,
};

#[derive(Debug, Deserialize)]
pub struct CategoryQuery {
    pub group_by_course: Option<bool>,
}

pub async fn get_folder_categories(
    Query(query): Query<CategoryQuery>,
) -> Json<serde_json::Value> {
    let categories = FolderCategory::get_all_categories();
    
    if query.group_by_course.unwrap_or(false) {
        let grouped = CourseGroupResponse::group_by_course(categories);
        Json(serde_json::json!({
            "success": true,
            "data": grouped
        }))
    } else {
        let responses: Vec<FolderCategoryResponse> = categories
            .into_iter()
            .map(FolderCategoryResponse::from)
            .collect();
            
        Json(serde_json::json!({
            "success": true,
            "data": FolderCategoryListResponse {
                categories: responses
            }
        }))
    }
}

pub async fn get_category_by_stage_code(
    axum::extract::Path(stage_code): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    match FolderCategory::categorize_by_stage_code(&stage_code) {
        Some(category) => {
            let response = FolderCategoryResponse::from(category);
            Json(serde_json::json!({
                "success": true,
                "data": response
            }))
        }
        None => {
            Json(serde_json::json!({
                "success": false,
                "error": "Stage code not found"
            }))
        }
    }
}