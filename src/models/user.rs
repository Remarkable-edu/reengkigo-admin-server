use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::dto::auth::AuthInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub user_id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUser {
    pub account_id: u32,
    pub account: String,
    pub role: String,
    pub agency_id: u32,
    pub academy_id: u32,
    pub is_active: bool,
}

pub const ACCOUNT_TYPE_HEAD_OFFICE: u32 = 1;
pub const ACCOUNT_TYPE_REGIONAL_MANAGER: u32 = 2;
pub const ACCOUNT_TYPE_DIRECTOR: u32 = 3;

impl AdminUser {
    fn map_account_type_to_role(account_type_id: u32) -> String {
        match account_type_id {
            ACCOUNT_TYPE_HEAD_OFFICE => "HEAD_OFFICE".to_string(),
            ACCOUNT_TYPE_REGIONAL_MANAGER => "REGIONAL_MANAGER".to_string(),
            ACCOUNT_TYPE_DIRECTOR => "DIRECTOR".to_string(),
            _ => "UNKNOWN".to_string(),
        }
    }

    pub fn is_director(&self) -> bool {
        self.role == "DIRECTOR"
    }

    pub fn is_head_office(&self) -> bool {
        self.role == "HEAD_OFFICE"
    }

    pub fn is_regional_manager(&self) -> bool {
        self.role == "REGIONAL_MANAGER"
    }

    pub fn can_access_admin(&self) -> bool {
        self.is_active && (self.is_head_office() || self.is_regional_manager())
    }

    pub fn can_access_director(&self) -> bool {
        self.is_active && self.is_director()
    }
}

impl From<AuthInfo> for AdminUser {
    fn from(auth_info: AuthInfo) -> Self {
        Self {
            account_id: auth_info.account_id,
            account: auth_info.account,
            role: Self::map_account_type_to_role(auth_info.account_type_id),
            agency_id: auth_info.agency_id,
            academy_id: auth_info.academy_id,
            is_active: auth_info.state == 1,
        }
    }
}
