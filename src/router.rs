use std::sync::Arc;

use crate::{
    app_state::AppState,
    oauth::OAuth,
    places::search::{DynPlacesSearch, Search},
    repository::{
        places::{DynPlacesRepo, PlacesRepository},
        user::{DynUserRepo, UserRepository},
    },
    routes::{
        auth::{auth, authenticate, refresh_token, verify_phone},
        ratings::search_for_place,
    },
    sms::{DynSMSVerify, SMSVerify},
};
use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};
use tokio::sync::Mutex;

pub fn create_router<U, P, S, V>(
    user_repo: U,
    places_repo: P,
    sms_verify: V,
    places_search: S,
    oauth: OAuth,
) -> Router
where
    U: UserRepository,
    P: PlacesRepository,
    S: Search,
    V: SMSVerify,
{
    let user_repo = Arc::new(Mutex::new(user_repo)) as DynUserRepo;
    let places_repo = Arc::new(Mutex::new(places_repo)) as DynPlacesRepo;
    let sms_verify = Arc::new(sms_verify) as DynSMSVerify;
    let places_search = Arc::new(Mutex::new(places_search)) as DynPlacesSearch;
    let app_state = AppState {
        user_repo,
        places_repo,
        sms_verify,
        places_search,
        oauth,
    };

    Router::new()
        .route("/search-places", get(search_for_place))
        .layer(middleware::from_fn_with_state(app_state.clone(), auth))
        .route("/authenticate", put(authenticate))
        .route("/verify-phone", post(verify_phone))
        .route("/refresh-token", post(refresh_token))
        .with_state(app_state)
}
