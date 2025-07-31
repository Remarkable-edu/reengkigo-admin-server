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
        // Extract token from headers (Authorization header or Cookie)
        let token = match extract_token_from_headers(&headers) {
            Some(token) => token,
            None => {
                tracing::debug!("No auth token found in headers");
                return Ok(create_unauthorized_response(&request));
            }
        };

        // Validate token using AuthService
        let auth_service = AuthService::new();
        let claims = match auth_service.validate_token(&token) {
            Ok(claims) => {
                tracing::debug!("Token validation successful for user: {}", claims.username);
                claims
            },
            Err(e) => {
                tracing::warn!("Token validation failed: {}", e);
                return Ok(create_unauthorized_response(&request));
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

        Ok(create_forbidden_response(&request))
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

        Ok(create_forbidden_response(&request))
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

        Ok(create_forbidden_response(&request))
    }
}

pub fn extract_token_from_headers(headers: &HeaderMap) -> Option<String> {
    // Try Authorization header first
    if let Some(auth_header) = headers.get(header::AUTHORIZATION) {
        if let Ok(header_str) = auth_header.to_str() {
            if header_str.starts_with("Bearer ") {
                let token = header_str[7..].to_string();
                tracing::debug!("Found Bearer token in Authorization header");
                return Some(token);
            }
        }
    }
    
    // Try Cookie header as fallback
    if let Some(cookie_header) = headers.get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            tracing::debug!("Parsing cookies: {}", cookie_str);
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if cookie.starts_with("auth_token=") {
                    let token = cookie[11..].to_string();
                    tracing::debug!("Found auth_token in cookies");
                    return Some(token);
                }
            }
        }
    }
    
    tracing::debug!("No token found in headers or cookies");
    None
}

fn is_api_request(request: &Request) -> bool {
    request.uri().path().starts_with("/api/") ||
    request.headers().get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|accept| accept.contains("application/json"))
        .unwrap_or(false)
}

fn create_unauthorized_response(request: &Request) -> Response {
    if is_api_request(request) {
        // Return JSON response for API requests
        let body = json!({
            "error": "UNAUTHORIZED",
            "message": "Authentication required"
        });
        
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body.to_string().into())
            .unwrap()
    } else {
        // Redirect to login page for browser requests
        Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/login")
            .header(header::SET_COOKIE, "auth_token=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0")
            .header(header::SET_COOKIE, "auth_status=; SameSite=Lax; Path=/; Max-Age=0")
            .body("".into())
            .unwrap()
    }
}

fn create_forbidden_response(request: &Request) -> Response {
    if is_api_request(request) {
        // Return JSON response for API requests
        let body = json!({
            "error": "FORBIDDEN",
            "message": "Insufficient permissions"
        });
        
        Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body.to_string().into())
            .unwrap()
    } else {
        // Redirect to login page for browser requests
        Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/login")
            .header(header::SET_COOKIE, "auth_token=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0")
            .header(header::SET_COOKIE, "auth_status=; SameSite=Lax; Path=/; Max-Age=0")
            .body("".into())
            .unwrap()
    }
}

// Helper function to get current user from request
pub fn get_current_user(request: &Request) -> Option<&AdminUser> {
    request.extensions().get::<AdminUser>()
}

