use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderCategory {
    pub course_name: String,
    pub stage_name: String,
    pub stage_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CourseType {
    MainCourse,
    Extension,
    Teenz,
    Supplementary,
}

impl FolderCategory {
    pub fn new(course_name: String, stage_name: String, stage_code: String) -> Self {
        Self {
            course_name,
            stage_name,
            stage_code,
        }
    }

    pub fn get_course_type(&self) -> CourseType {
        match self.course_name.as_str() {
            "링키영어 메인코스" => CourseType::MainCourse,
            "익스텐션 코스" => CourseType::Extension,
            "TEENZ 코스" => CourseType::Teenz,
            "Supplimentary 코스" => CourseType::Supplementary,
            _ => CourseType::MainCourse, // default
        }
    }

    pub fn categorize_by_stage_code(stage_code: &str) -> Option<Self> {
        // 실제 폴더명들을 기반으로 한 매핑
        match stage_code {
            // Main Course - U 시리즈 (Unit 기반)
            "U1B" | "U1G" | "U1O" | "U1P" | "U1O Full Story" | "U1O Read All" => {
                Some(Self::new("링키영어 메인코스".to_string(), "Unit 1".to_string(), stage_code.to_string()))
            },
            
            // A 시리즈 (Adventure 기반)
            "A4O" | "A4R" | "A4Y" | "A7P" => {
                Some(Self::new("링키영어 메인코스".to_string(), "Adventure".to_string(), stage_code.to_string()))
            },
            
            // B 시리즈 (Book 기반)
            "B8O" | "B8R" | "B8Y" => {
                Some(Self::new("링키영어 메인코스".to_string(), "Book".to_string(), stage_code.to_string()))
            },
            
            // E 시리즈 (English 기반)
            "E2O" | "E2R" | "E2Y" => {
                Some(Self::new("링키영어 메인코스".to_string(), "English".to_string(), stage_code.to_string()))
            },
            
            // J 시리즈 (Junior 기반)
            "J2O" | "J2R" => {
                Some(Self::new("익스텐션 코스".to_string(), "Junior".to_string(), stage_code.to_string()))
            },
            
            // K 시리즈 (Kids 기반)
            "K6O" | "K6R" | "K6Y" => {
                Some(Self::new("TEENZ 코스".to_string(), "Kids".to_string(), stage_code.to_string()))
            },
            
            // M, R 시리즈 (Phonics 기반)
            "M3P" | "R1P" | "R1R" | "R5P" => {
                Some(Self::new("Supplementary 코스".to_string(), "Phonics".to_string(), stage_code.to_string()))
            },
            
            _ => None,
        }
    }

    pub fn get_all_categories() -> Vec<Self> {
        vec![
            // Main Course
            Self::new("링키영어 메인코스".to_string(), "Stage1-1".to_string(), "ST1-1".to_string()),
            Self::new("링키영어 메인코스".to_string(), "Stage1-2".to_string(), "ST1-2".to_string()),
            Self::new("링키영어 메인코스".to_string(), "Stage2-1".to_string(), "ST2-1".to_string()),
            Self::new("링키영어 메인코스".to_string(), "Stage2-2".to_string(), "ST2-2".to_string()),
            Self::new("링키영어 메인코스".to_string(), "Stage3".to_string(), "ST3".to_string()),
            
            // Extension Course
            Self::new("익스텐션 코스".to_string(), "JELLY".to_string(), "JEL".to_string()),
            Self::new("익스텐션 코스".to_string(), "JUICE".to_string(), "JUI".to_string()),
            
            // Teenz Course
            Self::new("TEENZ 코스".to_string(), "TEENZ1-1".to_string(), "TZ1-1".to_string()),
            Self::new("TEENZ 코스".to_string(), "TEENZ1-2".to_string(), "TZ1-2".to_string()),
            Self::new("TEENZ 코스".to_string(), "TEENZ2-1".to_string(), "TZ2-1".to_string()),
            Self::new("TEENZ 코스".to_string(), "TEENZ2-2".to_string(), "TZ2-2".to_string()),
            Self::new("TEENZ 코스".to_string(), "TEENZ PHONICS".to_string(), "TZP".to_string()),
            
            // Supplementary Course
            Self::new("Supplimentary 코스".to_string(), "Reengki Phonics".to_string(), "RKP".to_string()),
            Self::new("Supplimentary 코스".to_string(), "Alphabet".to_string(), "ALP".to_string()),
        ]
    }
}