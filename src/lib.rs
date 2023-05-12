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
use places::search::Search;
use repository::{places::PlacesRepository, user::UserRepository};
use router::create_router;
use sms::SMSVerify;

pub async fn run<U: UserRepository, P: PlacesRepository, V: SMSVerify, S: Search>(
    user_repo: U,
    places_repo: P,
    sms_verify: V,
    places_search: S,
    oauth: OAuth,
) {
    let app = create_router(user_repo, places_repo, sms_verify, places_search, oauth);
    let address = SocketAddr::from(([0, 0, 0, 0], 8080));

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
