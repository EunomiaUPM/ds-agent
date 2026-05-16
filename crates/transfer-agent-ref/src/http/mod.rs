use std::sync::Arc;

use axum::{Router, middleware};
use common::auth::middleware::{TokenValidator, auth_middleware};

pub(crate) mod extractors;
pub(crate) mod transfer_message_router;
pub(crate) mod transfer_process_router;

pub(crate) fn build_router(
    token_validator: Arc<dyn TokenValidator>,
    process_router: Router,
    message_router: Router,
) -> Router {
    Router::new()
        .merge(process_router)
        .merge(message_router)
        .route_layer(middleware::from_fn_with_state(
            token_validator,
            auth_middleware,
        ))
}
