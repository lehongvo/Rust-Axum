use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::{
    Json,
    body::Body,
    extract::State,
    http::{Request, header},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{config::AppConfig, error::AppError, state::AppState};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
}

#[utoipa::path(
    post,
    path = "/login",
    tag = "api",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    if body.username != state.config.admin_user || body.password != state.config.admin_pass {
        return Err(AppError::Unauthorized);
    }

    let token = issue_token(&state.config)?;
    Ok(Json(LoginResponse { token }))
}

pub fn issue_token(config: &AppConfig) -> Result<String, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs() as usize;
    let exp = now + 60 * 60; // 1h expiry

    let claims = Claims {
        sub: config.admin_user.clone(),
        iat: now,
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|_| AppError::Unauthorized)
}

pub fn verify_token(config: &AppConfig, token: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized)
}

pub async fn require_api_key(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let api_key = req.headers().get("x-api-key").and_then(|v| v.to_str().ok());

    match api_key {
        Some(key) if key == state.config.api_key => Ok(next.run(req).await),
        _ => Err(AppError::Unauthorized),
    }
}

pub async fn require_jwt(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let token = auth_header
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    verify_token(&state.config, token)?;
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, http::Request, routing::get};
    use sea_orm::{DatabaseBackend, MockDatabase};
    use tower::ServiceExt;

    use crate::{config::AppConfig, middleware::rate_limit::RateState, state::AppState};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn config() -> AppConfig {
        AppConfig {
            database_url: "postgres://localhost/test".into(),
            app_port: 3000,
            api_key: "k".into(),
            jwt_secret: "secret".into(),
            admin_user: "admin".into(),
            admin_pass: "pass".into(),
            rate_limit_per_minute: 10,
        }
    }

    #[test]
    fn token_roundtrip() {
        let cfg = config();
        let token = issue_token(&cfg).unwrap();
        let claims = verify_token(&cfg, &token).unwrap();
        assert_eq!(claims.sub, "admin");
    }

    #[test]
    fn token_invalid_secret() {
        let cfg = config();
        let token = issue_token(&cfg).unwrap();
        let mut other = cfg.clone();
        other.jwt_secret = "wrong".into();
        assert!(verify_token(&other, &token).is_err());
    }

    fn mock_state() -> AppState {
        let db = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
        AppState {
            db: Arc::new(db),
            config: config(),
            rate_limiter: Arc::new(Mutex::new(RateState::new())),
        }
    }

    #[tokio::test]
    async fn api_key_middleware() {
        let state = mock_state();
        let app = Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                require_api_key,
            ))
            .with_state(state.clone());

        // missing key -> 401
        let res = app
            .clone()
            .oneshot(Request::new(Body::empty()))
            .await
            .unwrap();
        assert_eq!(res.status(), axum::http::StatusCode::UNAUTHORIZED);

        // valid key -> 200
        let mut req = Request::new(Body::empty());
        req.headers_mut()
            .insert("x-api-key", state.config.api_key.parse().unwrap());
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn jwt_middleware() {
        let state = mock_state();
        let token = issue_token(&state.config).unwrap();
        let app = Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                require_jwt,
            ))
            .with_state(state.clone());

        // missing token -> 401
        let res = app
            .clone()
            .oneshot(Request::new(Body::empty()))
            .await
            .unwrap();
        assert_eq!(res.status(), axum::http::StatusCode::UNAUTHORIZED);

        // valid token -> 200
        let mut req = Request::new(Body::empty());
        req.headers_mut().insert(
            header::AUTHORIZATION,
            format!("Bearer {}", token).parse().unwrap(),
        );
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), axum::http::StatusCode::OK);
    }
}
