use axum::{
    http::{HeaderMap, StatusCode},
    Json,
};

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

pub async fn authenticate(
    Json(payload): Json<AuthenticateRequest>,
) -> Result<(), (StatusCode, String)> {
    println!("{}", payload.first_name);
    println!("{}", payload.last_name);
    println!("{}", payload.phone_number);
    if payload.first_name != "Andrew" {
        return Err((
            StatusCode::BAD_REQUEST,
            "Expected name to be Andrew".to_string(),
        ));
    }
    Ok(())
}

pub async fn verify_phone() -> StatusCode {
    StatusCode::OK
}
