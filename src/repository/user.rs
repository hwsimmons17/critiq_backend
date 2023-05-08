use std::sync::Arc;

use tokio::sync::Mutex;

#[derive(Clone)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
    pub phone_number: u64,
    pub is_verified: bool,
}

pub type DynUserRepo = Arc<Mutex<dyn UserRepository>>;

pub trait UserRepository: Send + Sync + 'static {
    fn create<'a>(&mut self, user: User) -> Result<User, String>;
    fn read<'a>(&self, phone_number: u64) -> Result<Vec<User>, String>;
    fn update<'a>(&mut self, user: User) -> Result<User, String>;
    fn delete<'a>(&mut self, phone_number: u64) -> Result<Option<User>, String>;
}
