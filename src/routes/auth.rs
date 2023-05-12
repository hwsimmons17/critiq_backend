use axum::{
    extract::State,
    http::{self, Request, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{app_state::AppState, oauth::OAuth, repository::user::User};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateRequest {
    first_name: String,
    last_name: String,
    phone_number: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenResponse {
    access_token: String,
    refresh_token: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenRequest {
    refresh_token: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyRequest {
    phone_number: String,
    code: u32,
}

#[axum_macros::debug_handler]
pub async fn authenticate(
    State(app_state): State<AppState>,
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

    match app_state
        .user_repo
        .lock()
        .await
        .create(User {
            first_name,
            last_name,
            phone_number,
            is_verified: false,
        })
        .await
    {
        Ok(_) => {}
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ))
        }
    };

    match app_state
        .sms_verify
        .send_verification_code(phone_number)
        .await
    {
        Ok(_) => Ok(()),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )),
    }
}

pub async fn verify_phone(
    State(app_state): State<AppState>,
    Json(payload): Json<VerifyRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, String)> {
    let phone_number: u64;
    match validate_phone_number(&payload.phone_number) {
        Ok(number) => phone_number = number,
        Err(e) => return Err((StatusCode::BAD_REQUEST, e)),
    };

    let mut user: User;
    match app_state.user_repo.lock().await.read(phone_number).await {
        Ok(u) => {
            if u.len() == 0 {
                return Err((StatusCode::BAD_REQUEST, "No user saved".to_string()));
            }
            if u.len() > 1 {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Duplicate users with same phone number".to_string(),
                ));
            }
            user = u[0].clone();
        }
        Err(e) => {
            println!("{}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ));
        }
    }

    match app_state
        .sms_verify
        .verify_code(phone_number, payload.code)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
            return Err((StatusCode::BAD_REQUEST, e));
        }
    }
    user.is_verified = true;
    match app_state.user_repo.lock().await.update(user.clone()).await {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ));
        }
    }

    let access_token: String;
    match app_state.oauth.generate_jwt(
        &user.first_name,
        &user.last_name,
        user.phone_number,
        user.is_verified,
    ) {
        Ok(token) => access_token = token,
        Err(e) => {
            println!("{}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ));
        }
    };

    let refresh_token: String;
    match app_state.oauth.generate_refresh_token(
        &user.first_name,
        &user.last_name,
        user.phone_number,
        user.is_verified,
    ) {
        Ok(token) => refresh_token = token,
        Err(e) => {
            println!("{}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ));
        }
    }

    Ok(Json(TokenResponse {
        access_token,
        refresh_token,
    }))
}

pub async fn refresh_token(
    State(app_state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, String)> {
    let refresh_token = payload.refresh_token;
    let access_token: String;
    match app_state.oauth.refresh_token(&refresh_token) {
        Ok(token) => access_token = token,
        Err(_) => return Err((StatusCode::BAD_REQUEST, "Bad refresh token".to_string())),
    }

    Ok(Json(TokenResponse {
        access_token,
        refresh_token,
    }))
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

pub async fn auth<B>(
    State(app_state): State<AppState>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    match authorize_current_user(auth_header, app_state.oauth).await {
        Ok(current_user) => {
            req.extensions_mut().insert(current_user);
            Ok(next.run(req).await)
        }
        Err(e) => Err(e),
    }
}

async fn authorize_current_user(auth_header: &str, oauth: OAuth) -> Result<User, StatusCode> {
    match oauth.verify_jwt(auth_header) {
        Ok(u) => Ok(u),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
