use std::sync::Arc;

use axum::async_trait;

pub mod twilio;

pub type DynSMSVerify = Arc<dyn SMSVerify>;

#[async_trait]
pub trait SMSVerify: Send + Sync + 'static {
    async fn send_verification_code(&self, phone_number: u64) -> Result<(), String>;
    async fn verify_code(&self, phone_number: u64, verification_code: u32) -> Result<(), String>;
}
