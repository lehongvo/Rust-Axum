use std::time::Duration;

use axum::{
    Json, Router,
    body::Body,
    extract::State,
    middleware,
    routing::{get, post},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;
use utoipa::{ToSchema, OpenApi};

use crate::{
    auth::{login, require_api_key, require_jwt},
    error::AppError,
    models::{history, order, product, user},
    openapi::ApiDoc,
    state::AppState,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct NewProduct {
    pub name: String,
    pub price_cents: i64,
}

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct NewOrder {
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, ToSchema)]
pub struct UserRequest {
    pub email: String,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ProductResponse {
    pub id: Uuid,
    pub name: String,
    pub price_cents: i64,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct OrderResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub total_cents: i64,
}

pub fn router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/products", get(list_products).post(create_product))
        .route("/orders", post(create_order))
        .route(
            "/users",
            get(list_users)
                .post(create_user)
                .put(update_user)
                .delete(delete_user),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            enforce_rate_limit,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_jwt,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_api_key,
        ));

    Router::new()
        .route("/health", get(health))
        .route("/login", post(login))
        .merge(
            utoipa_swagger_ui::SwaggerUi::new("/docs")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
        )
        .merge(protected)
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "api",
    responses(
        (status = 200, description = "Health check", body = HealthResponse)
    )
)]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/products",
    tag = "api",
    responses(
        (status = 200, description = "List all products", body = Vec<ProductResponse>)
    )
)]
pub async fn list_products(
    State(state): State<AppState>,
) -> Result<Json<Vec<ProductResponse>>, AppError> {
    let products: Vec<ProductResponse> = product::Entity::find()
        .order_by_desc(product::Column::CreatedAt)
        .all(state.db.as_ref())
        .await?
        .into_iter()
        .map(|p| ProductResponse {
            id: p.id,
            name: p.name,
            price_cents: p.price_cents,
        })
        .collect();
    Ok(Json(products))
}

#[utoipa::path(
    post,
    path = "/products",
    tag = "api",
    request_body = NewProduct,
    responses(
        (status = 201, description = "Product created", body = ProductResponse)
    )
)]
pub async fn create_product(
    State(state): State<AppState>,
    Json(payload): Json<NewProduct>,
) -> Result<(axum::http::StatusCode, Json<ProductResponse>), AppError> {
    if payload.price_cents < 0 {
        return Err(AppError::BadRequest("price_cents must be positive".into()));
    }

    let model = product::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(payload.name),
        price_cents: Set(payload.price_cents),
        ..Default::default()
    };

    let product = model.insert(state.db.as_ref()).await?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(ProductResponse {
            id: product.id,
            name: product.name,
            price_cents: product.price_cents,
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/orders",
    tag = "api",
    request_body = NewOrder,
    responses(
        (status = 201, description = "Order created", body = OrderResponse)
    )
)]
pub async fn create_order(
    State(state): State<AppState>,
    Json(payload): Json<NewOrder>,
) -> Result<(axum::http::StatusCode, Json<OrderResponse>), AppError> {
    if payload.quantity <= 0 {
        return Err(AppError::BadRequest("quantity must be positive".into()));
    }

    let user = user::Entity::find_by_id(payload.user_id)
        .one(state.db.as_ref())
        .await?;
    let _ = user.ok_or_else(|| AppError::BadRequest("user not found".into()))?;

    let product = product::Entity::find_by_id(payload.product_id)
        .one(state.db.as_ref())
        .await?;

    let product = product.ok_or_else(|| AppError::BadRequest("product not found".into()))?;
    let total_cents = product.price_cents * i64::from(payload.quantity);

    let model = order::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(payload.user_id),
        product_id: Set(product.id),
        quantity: Set(payload.quantity),
        total_cents: Set(total_cents),
        ..Default::default()
    };

    let order = model.insert(state.db.as_ref()).await?;

    let hist = history::ActiveModel {
        id: Set(Uuid::new_v4()),
        order_id: Set(order.id),
        action: Set("created".to_string()),
        ..Default::default()
    };
    let _ = hist.insert(state.db.as_ref()).await?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(OrderResponse {
            id: order.id,
            user_id: order.user_id,
            product_id: order.product_id,
            quantity: order.quantity,
            total_cents: order.total_cents,
        }),
    ))
}

pub async fn enforce_rate_limit(
    State(state): State<AppState>,
    req: axum::http::Request<Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, AppError> {
    let mut guard = state.rate_limiter.lock().await;
    let allowed =
        guard.check_and_increment(state.config.rate_limit_per_minute, Duration::from_secs(60));
    drop(guard);

    if !allowed {
        return Err(AppError::Forbidden);
    }

    Ok(next.run(req).await)
}

#[utoipa::path(
    get,
    path = "/users",
    tag = "api",
    responses(
        (status = 200, description = "List all users", body = Vec<UserResponse>)
    )
)]
pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    let users = user::Entity::find()
        .order_by_desc(user::Column::CreatedAt)
        .all(state.db.as_ref())
        .await?;
    let mapped = users
        .into_iter()
        .map(|u| UserResponse {
            id: u.id,
            email: u.email,
        })
        .collect();
    Ok(Json(mapped))
}

#[utoipa::path(
    post,
    path = "/users",
    tag = "api",
    request_body = UserRequest,
    responses(
        (status = 201, description = "User created", body = UserResponse)
    )
)]
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<UserRequest>,
) -> Result<(axum::http::StatusCode, Json<UserResponse>), AppError> {
    if payload.email.trim().is_empty() {
        return Err(AppError::BadRequest("email required".into()));
    }

    let model = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(payload.email),
        ..Default::default()
    };

    let created = model.insert(state.db.as_ref()).await?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(UserResponse {
            id: created.id,
            email: created.email,
        }),
    ))
}

#[utoipa::path(
    put,
    path = "/users",
    tag = "api",
    request_body = UserRequest,
    responses(
        (status = 200, description = "User updated", body = UserResponse)
    )
)]
pub async fn update_user(
    State(state): State<AppState>,
    Json(payload): Json<UserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    if payload.email.trim().is_empty() {
        return Err(AppError::BadRequest("email required".into()));
    }

    // For simplicity, update first user by email match or return not found
    let existing = user::Entity::find()
        .filter(user::Column::Email.eq(payload.email.clone()))
        .one(state.db.as_ref())
        .await?
        .ok_or_else(|| AppError::BadRequest("user not found".into()))?;

    let mut active: user::ActiveModel = existing.into();
    active.email = Set(payload.email.clone());
    let updated = active.update(state.db.as_ref()).await?;

    Ok(Json(UserResponse {
        id: updated.id,
        email: updated.email,
    }))
}

#[utoipa::path(
    delete,
    path = "/users",
    tag = "api",
    request_body = UserRequest,
    responses(
        (status = 204, description = "User deleted")
    )
)]
pub async fn delete_user(
    State(state): State<AppState>,
    Json(payload): Json<UserRequest>,
) -> Result<axum::http::StatusCode, AppError> {
    let existing = user::Entity::find()
        .filter(user::Column::Email.eq(payload.email.clone()))
        .one(state.db.as_ref())
        .await?
        .ok_or_else(|| AppError::BadRequest("user not found".into()))?;

    let active: user::ActiveModel = existing.into();
    let _ = active.delete(state.db.as_ref()).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, http::Request, routing::get};
    use sea_orm::{DatabaseBackend, MockDatabase};
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tower::ServiceExt;

    fn state_with_limit(limit: u64) -> AppState {
        let db = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
        let cfg = crate::config::AppConfig {
            database_url: "postgres://localhost/test".into(),
            app_port: 3000,
            api_key: "k".into(),
            jwt_secret: "secret".into(),
            admin_user: "admin".into(),
            admin_pass: "pass".into(),
            rate_limit_per_minute: limit,
        };
        AppState {
            db: Arc::new(db),
            config: cfg,
            rate_limiter: Arc::new(Mutex::new(crate::middleware::rate_limit::RateState::new())),
        }
    }

    #[tokio::test]
    async fn rate_limit_blocks_after_limit() {
        let state = state_with_limit(1);
        let app = Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                enforce_rate_limit,
            ))
            .with_state(state.clone());

        let res1 = app
            .clone()
            .oneshot(Request::new(Body::empty()))
            .await
            .unwrap();
        assert_eq!(res1.status(), axum::http::StatusCode::OK);

        let res2 = app.oneshot(Request::new(Body::empty())).await.unwrap();
        assert_eq!(res2.status(), axum::http::StatusCode::FORBIDDEN);
    }
}
