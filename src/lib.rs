pub mod app_state;
pub mod repository;
mod router;
mod routes;

use std::net::SocketAddr;

use repository::user::UserRepository;
use router::create_router;

pub async fn run<U: UserRepository>(user_repo: U) {
    let app = create_router(user_repo);
    let address = SocketAddr::from(([0, 0, 0, 0], 8080));

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
