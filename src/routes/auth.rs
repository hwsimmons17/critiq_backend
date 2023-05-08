use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};

use crate::repository::user::{DynUserRepo, User};

#[derive(serde::Serialize)]
pub struct Message {
    message: String,
}

pub async fn handler(headers: HeaderMap) -> Json<Message> {
    let host = headers.get("host").unwrap().to_str().unwrap();

    Json(Message {
        message: host.to_string(),
    })
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateRequest {
    first_name: String,
    last_name: String,
    phone_number: String,
}

#[axum_macros::debug_handler]
pub async fn authenticate(
    State(user_repo): State<DynUserRepo>,
    Json(payload): Json<AuthenticateRequest>,
) -> Result<(), (StatusCode, String)> {
    let first_name: String;
    match validate_name(&payload.first_name) {
        Ok(name) => first_name = name,
        Err(e) => return Err((StatusCode::BAD_REQUEST, e)),
    };

    let last_name: String;
    match validate_name(&payload.last_name) {
        Ok(name) => last_name = name,
        Err(e) => return Err((StatusCode::BAD_REQUEST, e)),
    };

    let phone_number: u64;
    match validate_phone_number(&payload.phone_number) {
        Ok(number) => phone_number = number,
        Err(e) => return Err((StatusCode::BAD_REQUEST, e)),
    };

    match user_repo.lock().await.create(User {
        first_name,
        last_name,
        phone_number,
        is_verified: false,
    }) {
        Ok(_) => Ok(()),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )),
    }
}

fn validate_name(name: &str) -> Result<String, String> {
    let mut c = name.chars();
    match c.next() {
        Some(f) => Ok(f.to_uppercase().collect::<String>() + c.as_str()),
        None => Err("Name was empty".to_string()),
    }
}

fn validate_phone_number(phone_number: &str) -> Result<u64, String> {
    let mut numbers: Vec<u32> = vec![];
    let mut c = phone_number.chars();
    match c.next() {
        Some(n) => {
            if n != '(' {
                return Err("Phone number not valid".to_string());
            };
        }
        None => return Err("Phone number empty".to_string()),
    };

    for _ in 0..3 {
        match c.next() {
            Some(n) => {
                let dig = n.to_digit(10);
                match dig {
                    Some(d) => numbers.push(d),
                    None => return Err("Phone number not valid".to_string()),
                };
            }
            None => return Err("Phone number not valid".to_string()),
        };
    }

    match c.next() {
        Some(n) => {
            if n != ')' {
                return Err("Phone number not valid".to_string());
            };
        }
        None => return Err("Phone number empty".to_string()),
    };

    for _ in 0..3 {
        match c.next() {
            Some(n) => {
                let dig = n.to_digit(10);
                match dig {
                    Some(d) => numbers.push(d),
                    None => return Err("Phone number not valid".to_string()),
                };
            }
            None => return Err("Phone number not valid".to_string()),
        };
    }

    match c.next() {
        Some(n) => {
            if n != '-' {
                return Err("Phone number not valid".to_string());
            };
        }
        None => return Err("Phone number empty".to_string()),
    };

    for _ in 0..4 {
        match c.next() {
            Some(n) => {
                let dig = n.to_digit(10);
                match dig {
                    Some(d) => numbers.push(d),
                    None => return Err("Phone number not valid".to_string()),
                };
            }
            None => return Err("Phone number not valid".to_string()),
        };
    }

    if let Some(_) = c.next() {
        return Err("Phone number too long".to_string());
    }

    Ok(numbers.iter().fold(0, |acc, elem| acc * 10 + *elem as u64))
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenResponse {
    access_token: String,
    refresh_token: String,
    token_type: String,
}

pub async fn verify_phone() -> Result<Json<TokenResponse>, (StatusCode, String)> {
    Ok(Json(TokenResponse {
        access_token: "".to_string(),
        refresh_token: "".to_string(),
        token_type: "".to_string(),
    }))
}
