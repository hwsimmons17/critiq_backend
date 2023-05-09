use std::sync::Arc;

use axum::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
    pub phone_number: u64,
    pub is_verified: bool,
}

pub type DynUserRepo = Arc<Mutex<dyn UserRepository>>;

#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn create(&mut self, user: User) -> Result<User, String>;
    async fn read(&self, phone_number: u64) -> Result<Vec<User>, String>;
    async fn update(&mut self, user: User) -> Result<User, String>;
    async fn delete(&mut self, phone_number: u64) -> Result<Option<User>, String>;
}
