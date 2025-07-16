use anyhow::{Ok, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use reqwest;

use crate::{dto::auth::{ClaimsResponse, LoginRequest, LoginResponse}, models::user::AdminUser};


pub struct AuthService {
    jwt_secret: String,
    client: reqwest::Client,
}

impl AuthService {
    pub fn new() -> Self {
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret-key".to_string());
        let client = reqwest::Client::new();
        Self { jwt_secret, client }
    }

    pub fn generate_admin_token(&self, user: &AdminUser) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(24);

        let claims = ClaimsResponse {
            username: user.account.clone(),
            role: user.role.clone(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let token = encode( &Header::default(), &claims, &EncodingKey::from_secret(self.jwt_secret.as_ref()))?;

        Ok(token)
    }

    pub fn validate_token(&self, token: &str) -> Result<ClaimsResponse> {
        let token_data = decode::<ClaimsResponse>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        )?;

        Ok(token_data.claims)
    }

    pub async fn authenticate_user(&self, account: &str, password: &str) -> Result<Option<AdminUser>> {
        // Check for development mode first
        if let Some(dev_user) = self.try_dev_authentication(account, password) {
            return Ok(Some(dev_user));
        }

        // Fall back to external API authentication
        let login_request = LoginRequest {
            account: account.to_string(),
            password: password.to_string(),
        };

        let response = self.client
            .post("https://dev-admin.reengki.com/api/applogin")
            .json(&login_request)
            .send()
            .await?;

        if response.status().is_success() {
            let login_response: LoginResponse = response.json().await?;
            match login_response.auth {
                Some(auth_info) => Ok(Some(AdminUser::from(auth_info))),
                None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    fn try_dev_authentication(&self, account: &str, password: &str) -> Option<AdminUser> {
        let dev_mode = std::env::var("DEV_MODE").unwrap_or_else(|_| "false".to_string()) == "true";
        
        if !dev_mode {
            return None;
        }

        // Admin account
        if account == std::env::var("DEV_ADMIN_ACCOUNT").unwrap_or_else(|_| "admin".to_string()) 
            && password == std::env::var("DEV_ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".to_string()) {
            return Some(AdminUser {
                account_id: std::env::var("DEV_ADMIN_ACCOUNT_ID").unwrap_or_else(|_| "1".to_string()).parse().unwrap_or(1),
                account: account.to_string(),
                role: std::env::var("DEV_ADMIN_ROLE").unwrap_or_else(|_| "HEAD_OFFICE".to_string()),
                agency_id: std::env::var("DEV_ADMIN_AGENCY_ID").unwrap_or_else(|_| "1".to_string()).parse().unwrap_or(1),
                academy_id: std::env::var("DEV_ADMIN_ACADEMY_ID").unwrap_or_else(|_| "1".to_string()).parse().unwrap_or(1),
                is_active: std::env::var("DEV_ADMIN_IS_ACTIVE").unwrap_or_else(|_| "true".to_string()) == "true",
            });
        }

        // Director account
        if account == std::env::var("DEV_DIRECTOR_ACCOUNT").unwrap_or_else(|_| "director".to_string()) 
            && password == std::env::var("DEV_DIRECTOR_PASSWORD").unwrap_or_else(|_| "director123".to_string()) {
            return Some(AdminUser {
                account_id: std::env::var("DEV_DIRECTOR_ACCOUNT_ID").unwrap_or_else(|_| "2".to_string()).parse().unwrap_or(2),
                account: account.to_string(),
                role: std::env::var("DEV_DIRECTOR_ROLE").unwrap_or_else(|_| "DIRECTOR".to_string()),
                agency_id: std::env::var("DEV_DIRECTOR_AGENCY_ID").unwrap_or_else(|_| "2".to_string()).parse().unwrap_or(2),
                academy_id: std::env::var("DEV_DIRECTOR_ACADEMY_ID").unwrap_or_else(|_| "2".to_string()).parse().unwrap_or(2),
                is_active: std::env::var("DEV_DIRECTOR_IS_ACTIVE").unwrap_or_else(|_| "true".to_string()) == "true",
            });
        }

        // Regional Manager account
        if account == std::env::var("DEV_REGIONAL_ACCOUNT").unwrap_or_else(|_| "regional".to_string()) 
            && password == std::env::var("DEV_REGIONAL_PASSWORD").unwrap_or_else(|_| "regional123".to_string()) {
            return Some(AdminUser {
                account_id: std::env::var("DEV_REGIONAL_ACCOUNT_ID").unwrap_or_else(|_| "3".to_string()).parse().unwrap_or(3),
                account: account.to_string(),
                role: std::env::var("DEV_REGIONAL_ROLE").unwrap_or_else(|_| "REGIONAL_MANAGER".to_string()),
                agency_id: std::env::var("DEV_REGIONAL_AGENCY_ID").unwrap_or_else(|_| "3".to_string()).parse().unwrap_or(3),
                academy_id: std::env::var("DEV_REGIONAL_ACADEMY_ID").unwrap_or_else(|_| "3".to_string()).parse().unwrap_or(3),
                is_active: std::env::var("DEV_REGIONAL_IS_ACTIVE").unwrap_or_else(|_| "true".to_string()) == "true",
            });
        }

        None
    }

}