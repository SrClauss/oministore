pub mod admin;
pub mod audit_log;
pub mod cart;
pub mod cart_item;
pub mod category;
pub mod collection;
pub mod coupon;
pub mod customer;
pub mod inventory;
pub mod order;
pub mod product;
pub mod shipping;
pub mod store;
pub mod upload;
pub mod user;
pub mod warehouse;
pub mod webhooks;

use axum::Router;

pub fn router() -> Router {
    Router::new()
        .nest("/carts", cart::router())
        .nest("/cart-items", cart_item::router())
        .nest("/categories", category::router())
        .nest("/collections", collection::router())
        .nest("/coupons", coupon::router())
        .nest("/customers", customer::router())
        .nest("/inventory", inventory::router())
        .nest("/orders", order::router())
        .nest("/products", product::router())
        .nest("/shipping", shipping::router())
        .nest("/stores", store::router())
        .nest("/uploads", upload::router())
        .nest("/users", user::router())
        .nest("/warehouses", warehouse::router())
        .nest("/audit-logs", audit_log::router())
        .nest("/webhooks", webhooks::router())
        .nest("/admin", admin::router())
}
