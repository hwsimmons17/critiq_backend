pub mod app_state;
pub mod repository;
mod router;
mod routes;

use std::net::SocketAddr;

use app_state::AppState;
use axum::{
    http::{HeaderMap, StatusCode},
    Json, Router,
};
use repository::user::UserRepository;
use router::create_router;

pub struct App<U: UserRepository> {
    router: Router,
    user_repo: U,
}

pub async fn run<U: UserRepository>(app_state: AppState<U>) {
    let app = create_router(app_state);
    let address = SocketAddr::from(([0, 0, 0, 0], 8080));

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
