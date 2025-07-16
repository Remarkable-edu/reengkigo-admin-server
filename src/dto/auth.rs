use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub account: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthInfo {
    #[serde(rename = "AccountID")]
    pub account_id: u32,
    #[serde(rename = "AccountTypeID")]
    pub account_type_id: u32,
    #[serde(rename = "AgencyID")]
    pub agency_id: u32,
    #[serde(rename = "AcademyID")]
    pub academy_id: u32,
    #[serde(rename = "Account")]
    pub account: String,
    #[serde(rename = "State")]
    pub state: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub auth: Option<AuthInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaimsResponse {
    pub username: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}
