use serde::{Deserialize, Serialize};
use crate::models::folder_category::{FolderCategory, CourseType};

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderCategoryResponse {
    pub course_name: String,
    pub stage_name: String,
    pub stage_code: String,
    pub course_type: String,
}

impl From<FolderCategory> for FolderCategoryResponse {
    fn from(category: FolderCategory) -> Self {
        let course_type = match category.get_course_type() {
            CourseType::MainCourse => "main_course",
            CourseType::Extension => "extension",
            CourseType::Teenz => "teenz",
            CourseType::Supplementary => "supplementary",
        };

        Self {
            course_name: category.course_name,
            stage_name: category.stage_name,
            stage_code: category.stage_code,
            course_type: course_type.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderCategoryListResponse {
    pub categories: Vec<FolderCategoryResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CourseGroupResponse {
    pub course_name: String,
    pub course_type: String,
    pub stages: Vec<StageResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StageResponse {
    pub stage_name: String,
    pub stage_code: String,
}

impl CourseGroupResponse {
    pub fn group_by_course(categories: Vec<FolderCategory>) -> Vec<Self> {
        use std::collections::HashMap;

        let mut grouped: HashMap<String, Vec<FolderCategory>> = HashMap::new();
        
        for category in categories {
            grouped.entry(category.course_name.clone())
                .or_insert_with(Vec::new)
                .push(category);
        }

        grouped.into_iter()
            .map(|(course_name, categories)| {
                let course_type = if let Some(first_category) = categories.first() {
                    match first_category.get_course_type() {
                        CourseType::MainCourse => "main_course",
                        CourseType::Extension => "extension",
                        CourseType::Teenz => "teenz",
                        CourseType::Supplementary => "supplementary",
                    }
                } else {
                    "unknown"
                };

                let stages: Vec<StageResponse> = categories.into_iter()
                    .map(|category| StageResponse {
                        stage_name: category.stage_name,
                        stage_code: category.stage_code,
                    })
                    .collect();

                Self {
                    course_name,
                    course_type: course_type.to_string(),
                    stages,
                }
            })
            .collect()
    }
}