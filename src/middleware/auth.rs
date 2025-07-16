use axum::{
    extract::{Request},
    http::{header, HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use serde_json::json;

use crate::{
    models::user::AdminUser,
    services::auth::AuthService,
};

pub struct AuthMiddleware;

impl AuthMiddleware {
    pub async fn auth_middleware(
        headers: HeaderMap,
        mut request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        // Extract token from Authorization header
        let token = match extract_token_from_headers(&headers) {
            Some(token) => token,
            None => {
                return Ok(unauthorized_response());
            }
        };

        // Validate token using AuthService
        let auth_service = AuthService::new();
        let claims = match auth_service.validate_token(&token) {
            Ok(claims) => claims,
            Err(_) => {
                return Ok(unauthorized_response());
            }
        };

        // Create AdminUser from claims for role checking
        let admin_user = AdminUser {
            account_id: 0, // Not available in JWT claims
            account: claims.username.clone(),
            role: claims.role.clone(),
            agency_id: 0, // Not available in JWT claims
            academy_id: 0, // Not available in JWT claims
            is_active: true, // Token validation implies active
        };
        
        tracing::debug!("Created AdminUser: {:?}", admin_user);
        tracing::debug!("AdminUser can_access_admin: {}", admin_user.can_access_admin());

        // Store user info in request extensions for use in handlers
        request.extensions_mut().insert(admin_user);
        request.extensions_mut().insert(claims);

        // Continue to the next middleware/handler
        Ok(next.run(request).await)
    }

    pub async fn require_admin_role(
        _headers: HeaderMap,
        request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        // Check if user has admin role (HEAD_OFFICE or REGIONAL_MANAGER)
        
        if let Some(admin_user) = request.extensions().get::<AdminUser>() {
            tracing::debug!("require_admin_role: AdminUser found: {:?}", admin_user);
            tracing::debug!("require_admin_role: can_access_admin: {}", admin_user.can_access_admin());
            if admin_user.can_access_admin() {
                return Ok(next.run(request).await);
            }
        } else {
            tracing::debug!("require_admin_role: No AdminUser found in request extensions");
        }

        Ok(forbidden_response())
    }

    pub async fn require_director_role(
        _headers: HeaderMap,
        request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        // Check if user has director role
        if let Some(admin_user) = request.extensions().get::<AdminUser>() {
            if admin_user.can_access_director() {
                return Ok(next.run(request).await);
            }
        }

        Ok(forbidden_response())
    }

    pub async fn require_any_role(
        _headers: HeaderMap,
        request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        // Check if user has any valid role and is active
        if let Some(admin_user) = request.extensions().get::<AdminUser>() {
            if admin_user.is_active {
                return Ok(next.run(request).await);
            }
        }

        Ok(forbidden_response())
    }
}

fn extract_token_from_headers(headers: &HeaderMap) -> Option<String> {
    // Try Authorization header first
    if let Some(auth_header) = headers.get(header::AUTHORIZATION) {
        if let Ok(header_str) = auth_header.to_str() {
            if header_str.starts_with("Bearer ") {
                return Some(header_str[7..].to_string());
            }
        }
    }
    
    // Try Cookie header as fallback
    if let Some(cookie_header) = headers.get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if cookie.starts_with("auth_token=") {
                    return Some(cookie[11..].to_string());
                }
            }
        }
    }
    
    None
}

fn unauthorized_response() -> Response {
    let body = json!({
        "error": "UNAUTHORIZED",
        "message": "Authentication required"
    });
    
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(header::CONTENT_TYPE, "application/json")
        .body(body.to_string().into())
        .unwrap()
}

fn forbidden_response() -> Response {
    let body = json!({
        "error": "FORBIDDEN",
        "message": "Insufficient permissions"
    });
    
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(body.to_string().into())
        .unwrap()
}

// Helper function to get current user from request
pub fn get_current_user(request: &Request) -> Option<&AdminUser> {
    request.extensions().get::<AdminUser>()
}

