pub mod app_state;
pub mod geo;
pub mod oauth;
pub mod places;
pub mod repository;
mod router;
mod routes;
pub mod sms;

use std::net::SocketAddr;

use oauth::OAuth;
use repository::user::UserRepository;
use router::create_router;
use sms::SMSVerify;

pub async fn run<U: UserRepository, V: SMSVerify>(user_repo: U, sms_verify: V, oauth: OAuth) {
    let app = create_router(user_repo, sms_verify, oauth);
    let address = SocketAddr::from(([0, 0, 0, 0], 8080));

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
