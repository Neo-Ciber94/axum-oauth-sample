mod api;
mod pages;

pub use api::api_router;
pub use pages::pages_router;
pub use pages::error_handler_middleware;