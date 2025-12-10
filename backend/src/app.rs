use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    config::AppConfig, error::AppError, middleware::rate_limit::RateState, migrations,
    state::AppState,
};

pub async fn build_app_state() -> Result<AppState, AppError> {
    let config = AppConfig::from_env().map_err(AppError::BadRequest)?;
    let db = sea_orm::Database::connect(&config.database_url).await?;
    migrations::run(&db).await?;

    Ok(AppState {
        db: Arc::new(db),
        config: config.clone(),
        rate_limiter: Arc::new(Mutex::new(RateState::new())),
    })
}

pub fn build_router(state: AppState) -> axum::Router {
    crate::http::router(state)
}
