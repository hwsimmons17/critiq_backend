use axum::async_trait;

use crate::repository::user::{User, UserRepository};

#[derive(Clone)]
pub struct LocalUserRepository {
    users: Vec<User>,
}

impl LocalUserRepository {
    pub fn new() -> LocalUserRepository {
        return LocalUserRepository { users: Vec::new() };
    }
}

#[async_trait]
impl UserRepository for LocalUserRepository {
    async fn create(&mut self, user: User) -> Result<User, String> {
        self.users.push(user.clone());
        Ok(user)
    }

    async fn read(&self, phone_number: u64) -> Result<Vec<User>, String> {
        Ok(self
            .users
            .clone()
            .into_iter()
            .filter(|u| u.phone_number == phone_number)
            .collect())
    }

    async fn update(&mut self, user: User) -> Result<User, String> {
        self.users = self
            .users
            .clone()
            .into_iter()
            .map(|u| {
                if u.phone_number == user.phone_number {
                    return user.clone();
                };
                u
            })
            .collect();
        Ok(user)
    }

    async fn delete(&mut self, phone_number: u64) -> Result<Option<User>, String> {
        let mut user: Option<User> = None;
        self.users = self
            .users
            .clone()
            .into_iter()
            .filter(|u| {
                if u.phone_number == phone_number {
                    user = Some(u.clone());
                };
                u.phone_number != phone_number
            })
            .collect();
        Ok(user)
    }
}
