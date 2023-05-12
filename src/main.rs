mod sms;

use std::sync::Arc;

use critiq_backend::{
    oauth::OAuth, places::mapbox::search::MapboxSearchApi, repository::subabase::SupabaseRepo, run,
    sms::twilio::TwilioSMS,
};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let twilio_account_sid =
        std::env::var("TWILIO_ACCOUNT_SID").expect("TWILIO_ACCOUNT_SID must be set.");
    let twilio_service_sid =
        std::env::var("TWILIO_SERVICE_SID").expect("TWILIO_SERVICE_SID must be set.");
    let twilio_auth_token =
        std::env::var("TWILIO_AUTH_TOKEN").expect("TWILIO_AUTH_TOKEN must be set.");
    let jwt_key = std::env::var("JWT_KEY").expect("JWT_KEY must be set.");
    let supabase_url = std::env::var("SUPABASE_URL").expect("SUPABASE_URL must be set.");
    let supabase_api_key =
        std::env::var("SUPABASE_API_KEY").expect("SUPABASE_API_KEY must be set.");
    let mapbox_api_key = std::env::var("MAPBOX_API_KEY").expect("MAPBOX_API_KEY must be set.");
    let foursquare_api_key =
        std::env::var("FOURSQUARE_API_KEY").expect("FOURSQUARE_API_KEY must be set.");

    let user_repo = SupabaseRepo::new(&supabase_url, &supabase_api_key);
    let places_repo = SupabaseRepo::new(&supabase_url, &supabase_api_key);
    let sms_verify = TwilioSMS::new(&twilio_account_sid, &twilio_service_sid, &twilio_auth_token);
    let places_search = MapboxSearchApi::new(
        &mapbox_api_key,
        &foursquare_api_key,
        Arc::new(Mutex::new(SupabaseRepo::new(
            &supabase_url,
            &supabase_api_key,
        ))),
    );
    let oauth = OAuth::new(&jwt_key);

    run(user_repo, places_repo, sms_verify, places_search, oauth).await
}
