use std::sync::Arc;

use crate::{
    app_state::AppState,
    repository::{
        local::user,
        user::{DynUserRepo, UserRepository},
    },
    routes::auth::{authenticate, handler, verify_phone},
};
use axum::{
    routing::{get, post, put},
    Router,
};
use tokio::sync::Mutex;

pub fn create_router<U: UserRepository>(user_repo: U) -> Router {
    let user_repo = Arc::new(Mutex::new(user_repo)) as DynUserRepo;

    Router::new()
        .route("/", get(handler))
        .route("/authenticate", put(authenticate))
        .route("/verify-phone", post(verify_phone))
        .with_state(user_repo)
}
