use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::AppConfig;
use crate::middleware::rate_limit::RateState;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub config: AppConfig,
    pub rate_limiter: Arc<Mutex<RateState>>,
}
