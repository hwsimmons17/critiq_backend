mod sms;

use critiq_backend::{
    oauth::OAuth, repository::local::user::LocalUserRepository, run, sms::twilio::TwilioSMS,
};

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

    let user_repo = LocalUserRepository::new();
    let sms_verify = TwilioSMS::new(&twilio_account_sid, &twilio_service_sid, &twilio_auth_token);
    let oauth = OAuth::new(&jwt_key);

    run(user_repo, sms_verify, oauth).await
}
