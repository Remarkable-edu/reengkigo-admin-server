use axum::{
    extract::State, 
    response::{Html, Json}, 
    Form, 
    http::{StatusCode, header::{SET_COOKIE, HeaderMap}},
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{AppState, services::auth::AuthService, models::user::AdminUser};

pub async fn login_page() -> Html<&'static str> {
    Html(include_str!("../templates/login.html"))
}

#[derive(Deserialize)]
pub struct LoginForm {
    account: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginSuccess {
    pub success: bool,
    pub user: AdminUser,
    pub token: String,
}

#[derive(Serialize)]
pub struct LoginError {
    pub success: bool,
    pub message: String,
}

pub async fn login_handler(
    State(_app_state): State<AppState>,
    Form(login_form): Form<LoginForm>,
) -> Result<(HeaderMap, Json<LoginSuccess>), (StatusCode, Json<LoginError>)> {
    info!("Login attempt for account: {}", login_form.account);
    let auth_service = AuthService::new();
    
    match auth_service.authenticate_user(&login_form.account, &login_form.password).await {
        Ok(Some(admin_user)) => {
            // Generate JWT token for the authenticated user
            match auth_service.generate_admin_token(&admin_user) {
                Ok(token) => {
                    // Create cookie headers
                    let mut headers = HeaderMap::new();
                    // HttpOnly cookie for actual authentication
                    let auth_cookie = format!("auth_token={}; HttpOnly; SameSite=Strict; Path=/; Max-Age=86400", token);
                    headers.insert(SET_COOKIE, auth_cookie.parse().unwrap());
                    
                    // Non-HttpOnly cookie for JavaScript to check auth status
                    let status_cookie = format!("auth_status=authenticated; SameSite=Strict; Path=/; Max-Age=86400");
                    headers.append(SET_COOKIE, status_cookie.parse().unwrap());
                    
                    Ok((headers, Json(LoginSuccess {
                        success: true,
                        user: admin_user,
                        token,
                    })))
                },
                Err(e) => {
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(LoginError {
                            success: false,
                            message: format!("Token generation error: {}", e),
                        }),
                    ))
                }
            }
        },
        Ok(None) => {
            Err((
                StatusCode::UNAUTHORIZED,
                Json(LoginError {
                    success: false,
                    message: "Invalid credentials".to_string(),
                }),
            ))
        },
        Err(e) => {
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginError {
                    success: false,
                    message: format!("Authentication error: {}", e),
                }),
            ))
        }
    }
}