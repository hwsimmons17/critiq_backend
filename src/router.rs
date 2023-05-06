use crate::{
    app_state::AppState,
    repository::user::UserRepository,
    routes::auth::{authenticate, handler, verify_phone},
};
use axum::{
    routing::{get, post, put},
    Router,
};

pub fn create_router<U: UserRepository>(app_state: AppState<U>) -> Router {
    Router::new()
        .route("/", get(handler))
        .route("/authenticate", put(authenticate))
        .route("/verify-phone", post(verify_phone))
        .with_state(app_state)
}
