pub mod routes;
pub use routes::{
    HealthResponse, NewOrder, NewProduct, OrderResponse, ProductResponse, UserRequest,
    UserResponse, create_order, create_product, create_user, delete_user, health, list_products,
    list_users, update_user,
};

pub use routes::router;
