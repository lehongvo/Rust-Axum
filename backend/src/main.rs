mod app;
mod auth;
mod config;
mod error;
mod http;
mod middleware;
mod migrations;
mod models;
mod openapi;
mod state;

use std::net::SocketAddr;

use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{app::build_app_state, app::build_router, error::AppError};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = build_app_state().await?;
    let app = build_router(state.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], state.config.app_port));
    tracing::info!("listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await.map_err(AppError::from)
}
