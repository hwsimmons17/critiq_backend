use axum::async_trait;
use base64::{engine::general_purpose, Engine};
use reqwest::{StatusCode, Url};

use super::SMSVerify;

#[derive(Clone)]
pub struct TwilioSMS {
    account_sid: String,
    service_sid: String,
    auth_token: String,
}

impl TwilioSMS {
    pub fn new(account_sid: &str, service_sid: &str, auth_token: &str) -> TwilioSMS {
        return TwilioSMS {
            account_sid: account_sid.clone().to_owned(),
            service_sid: service_sid.clone().to_owned(),
            auth_token: auth_token.clone().to_owned(),
        };
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct VerificationCheckResponse {
    status: String,
}

#[async_trait]
impl SMSVerify for TwilioSMS {
    async fn send_verification_code(&self, phone_number: u64) -> Result<(), String> {
        let mut url = "https://verify.twilio.com/v2/Services"
            .parse::<Url>()
            .unwrap();
        url.path_segments_mut()
            .map_err(|_| "cannot be base")
            .unwrap()
            .push(&self.service_sid)
            .push("Verifications");

        let client = reqwest::Client::new();
        let params = [
            ("To", &*phone_number.format_phone_number()),
            ("Channel", "sms"),
        ];
        let auth_header =
            general_purpose::STANDARD.encode(format!("{}:{}", self.account_sid, self.auth_token));
        match client
            .post(url)
            .form(&params)
            .header("Authorization", format!("Basic {}", auth_header))
            .send()
            .await
        {
            Ok(res) => {
                if res.status() != StatusCode::CREATED {
                    return Err("Error sending to Twillio".to_string());
                };
                Ok(())
            }
            Err(_) => Err("Error sending to Twilio".to_string()),
        }
    }
    async fn verify_code(&self, phone_number: u64, verification_code: u32) -> Result<(), String> {
        let mut url = "https://verify.twilio.com/v2/Services"
            .parse::<Url>()
            .unwrap();
        url.path_segments_mut()
            .map_err(|_| "cannot be base")
            .unwrap()
            .push(&self.service_sid)
            .push("VerificationCheck");

        let client = reqwest::Client::new();
        let params = [
            ("To", &*phone_number.format_phone_number()),
            ("Code", &*verification_code.format_verification_code()),
        ];
        let auth_header =
            general_purpose::STANDARD.encode(format!("{}:{}", self.account_sid, self.auth_token));
        match client
            .post(url)
            .form(&params)
            .header("Authorization", format!("Basic {}", auth_header))
            .send()
            .await
        {
            Ok(res) => {
                if res.status() != StatusCode::OK {
                    println!("{}", res.status());
                    return Err("Error sending to Twillio".to_string());
                };
                match res.text().await {
                    Ok(t) => {
                        println!("{}", t);
                        let body: Result<VerificationCheckResponse, serde_json::Error> =
                            serde_json::from_str(&t);
                        match body {
                            Ok(body) => {
                                if body.status != "approved".to_string() {
                                    return Err("Status not accepted".to_string());
                                }
                                Ok(())
                            }
                            Err(_) => Err("Error parsing JSON".to_string()),
                        }
                    }
                    Err(_) => todo!(),
                }
            }
            Err(_) => Err("Error sending to Twilio".to_string()),
        }
    }
}

pub trait PhoneNumber {
    fn format_phone_number(&self) -> String;
}

impl PhoneNumber for u64 {
    fn format_phone_number(&self) -> String {
        "+1".to_string() + &self.to_string()
    }
}

trait VerificationCode {
    fn format_verification_code(self) -> String;
}

impl VerificationCode for u32 {
    fn format_verification_code(self) -> String {
        if self < 10_000 {
            return format!("0{}", self.to_string());
        }

        self.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sms::SMSVerify;

    #[tokio::test]
    async fn test_send_verification_code() {
        let twilio_sms = TwilioSMS::new(
            "ACad6d83ed33e323da8f9713578aa7581d",
            "VA2ad3f46d9de8aec5442fdf86540a96db",
            "d7f58d7ec89e0eb3ea2f604d4e0ad960",
        );

        let res = twilio_sms.send_verification_code(2028098680).await;
        assert_eq!(res, Ok(()))
    }

    #[tokio::test]
    async fn test_verify_code() {
        let twilio_sms = TwilioSMS::new(
            "ACad6d83ed33e323da8f9713578aa7581d",
            "VA2ad3f46d9de8aec5442fdf86540a96db",
            "d7f58d7ec89e0eb3ea2f604d4e0ad960",
        );

        let res = twilio_sms.verify_code(2028098680, 06329).await;
        assert_eq!(res, Ok(()))
    }
}
