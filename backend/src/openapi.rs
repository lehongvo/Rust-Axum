use utoipa::OpenApi;

use crate::http::routes::{
    HealthResponse, NewOrder, NewProduct, OrderResponse, ProductResponse, UserRequest, UserResponse,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::http::routes::health,
        crate::http::routes::login,
        crate::http::routes::list_products,
        crate::http::routes::create_product,
        crate::http::routes::create_order,
        crate::http::routes::list_users,
        crate::http::routes::create_user,
        crate::http::routes::update_user,
        crate::http::routes::delete_user,
    ),
    components(
        schemas(
            HealthResponse,
            NewProduct,
            ProductResponse,
            NewOrder,
            OrderResponse,
            UserRequest,
            UserResponse
        )
    ),
    tags(
        (name = "api", description = "E-commerce API")
    )
)]
pub struct ApiDoc;
