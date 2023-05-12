use crate::{repository::user::User, sms::twilio::PhoneNumber};
use chrono::{DateTime, Duration, Utc};
use hmac::Hmac;
use jwt::{Error, SignWithKey, VerifyWithKey};
use sha2::Sha256;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct OAuth {
    key: Hmac<Sha256>,
}

impl OAuth {
    pub fn new(key: &str) -> Self {
        let key = bs58::decode(key).into_vec().unwrap();
        let key: Hmac<Sha256> = hmac::Mac::new_from_slice(&key).unwrap();
        OAuth { key }
    }

    pub fn generate_jwt(
        &self,
        first_name: &str,
        last_name: &str,
        phone_number: u64,
        is_verified: bool,
    ) -> Result<String, Error> {
        let mut claims = BTreeMap::new();
        claims.insert("first_name", first_name.to_string());
        claims.insert("last_name", last_name.to_string());
        claims.insert("phone_number", phone_number.format_phone_number());
        claims.insert("is_verified", is_verified.to_string());
        claims.insert("exp", (Utc::now() + Duration::days(1)).to_rfc3339());

        claims.sign_with_key(&self.key)
    }

    pub fn verify_jwt(&self, token_str: &str) -> Result<User, String> {
        let claims: BTreeMap<String, String>;
        match token_str.verify_with_key(&self.key) {
            Ok(c) => claims = c,
            Err(_) => return Err("Error verifying token string".to_string()),
        };

        match DateTime::parse_from_rfc3339(&claims["exp"]) {
            Ok(exp) => {
                if exp < Utc::now() {
                    return Err("Token expired".to_string());
                }
            }
            Err(_) => {
                return Err("Error parsing expiry date. Must be in RFC3339 format".to_string())
            }
        };

        let phone_number: u64;
        let is_verified: bool;
        match claims["phone_number"].parse() {
            Ok(number) => phone_number = number,
            Err(_) => return Err("Error parsing phone number".to_string()),
        }
        match claims["is_verified"].parse() {
            Ok(verified) => is_verified = verified,
            Err(_) => return Err("Error parsing is_verified".to_string()),
        }

        Ok(User {
            first_name: claims["first_name"].clone(),
            last_name: claims["last_name"].clone(),
            phone_number,
            is_verified,
        })
    }

    pub fn generate_refresh_token(
        &self,
        first_name: &str,
        last_name: &str,
        phone_number: u64,
        is_verified: bool,
    ) -> Result<String, Error> {
        let mut claims = BTreeMap::new();
        claims.insert("first_name", first_name.to_string());
        claims.insert("last_name", last_name.to_string());
        claims.insert("phone_number", phone_number.format_phone_number());
        claims.insert("is_verified", is_verified.to_string());

        claims.sign_with_key(&self.key)
    }

    pub fn refresh_token(&self, refresh_token: &str) -> Result<String, Error> {
        let mut claims: BTreeMap<String, String>;
        match refresh_token.verify_with_key(&self.key) {
            Ok(c) => claims = c,
            Err(_) => return Err(Error::InvalidSignature),
        };
        claims.insert(
            "exp".to_string(),
            (Utc::now() + Duration::days(1)).to_rfc3339(),
        );

        claims.sign_with_key(&self.key)
    }
}

// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn test_oauth() {
//         let key = "5atKdFrP3CcuCocV42qJvnCTQ7zsuHfuFkMHmHiZrZxK16K4vfa2NabpRjaMKn5M91fKnk5xVGhxNV"
//             .to_string();

//         let oauth = OAuth::new(&key);

//         let access_token = oauth
//             .generate_jwt("Hunter", "Simmons", 2028098680, true)
//             .unwrap();

//         let refresh_token = oauth
//             .generate_refresh_token("Hunter", "Simmons", 2028098680, true)
//             .unwrap();
//         let new_access_token = oauth.refresh_token(&refresh_token).unwrap();

//         let access_token_verified =
//             oauth.verify_jwt(&new_access_token, "Hunter", "Simmons", 2028098680, true);
//         assert_eq!(access_token_verified, Ok(()));
//     }
// }
