use std::sync::Arc;

use crate::{
    app_state::AppState,
    oauth::OAuth,
    repository::user::{DynUserRepo, UserRepository},
    routes::auth::{authenticate, verify_phone},
    sms::{DynSMSVerify, SMSVerify},
};
use axum::{
    routing::{post, put},
    Router,
};
use tokio::sync::Mutex;

pub fn create_router<U: UserRepository, V: SMSVerify>(
    user_repo: U,
    sms_verify: V,
    oauth: OAuth,
) -> Router {
    let user_repo = Arc::new(Mutex::new(user_repo)) as DynUserRepo;
    let sms_verify = Arc::new(sms_verify) as DynSMSVerify;
    let app_state = AppState {
        user_repo,
        sms_verify,
        oauth,
    };

    Router::new()
        .route("/authenticate", put(authenticate))
        .route("/verify-phone", post(verify_phone))
        .with_state(app_state)
}
