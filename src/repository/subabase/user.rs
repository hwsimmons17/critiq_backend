use axum::async_trait;
use reqwest::StatusCode;

use crate::repository::user::{User, UserRepository};

use super::SupabaseRepo;

#[async_trait]
impl UserRepository for SupabaseRepo {
    async fn create(&mut self, user: User) -> Result<User, String> {
        match  self.client
            .from("users")
            .insert(format!(
                r#"[{{"phone_number": "{}", "first_name": "{}", "last_name": "{}", "is_verified": "{}"}}]"#,
                user.phone_number.to_string(),
                user.first_name,
                user.last_name,
                user.is_verified
            ))
            .execute()
            .await {
                Ok(r) => {
                    if r.status() == StatusCode::CREATED {
                        return Ok(user)
                    }
                    if r.status() == StatusCode::CONFLICT {
                        return unwrap_read_user(self.read(user.phone_number).await);
                    }
            
                    return Err("User not created".to_string())
                }
                Err(_) => {
                    return Err("User not created".to_string())
                }
            }
    }

    async fn read(&self, phone_number: u64) -> Result<Vec<User>, String> {
        match self
            .client
            .from("users")
            .eq("phone_number", phone_number.to_string())
            .select("*")
            .execute()
            .await
        {
            Ok(r) => match r.text().await {
                Ok(t) => {
                    let body: Result<Vec<User>, serde_json::Error> = serde_json::from_str(&t);
                    match body {
                        Ok(b) => Ok(b),
                        Err(_) => Err("Could not read users".to_string()),
                    }
                }
                Err(_) => Err("Could not read users".to_string()),
            },
            Err(_) => return Err("Could not read users".to_string()),
        }
    }

    async fn update(&mut self, user: User) -> Result<User, String> {
        match self
            .client
            .from("users")
            .eq("phone_number", user.phone_number.to_string())
            .update(format!(
                r#"[{{"first_name": "{}", "last_name": "{}", "is_verified": "{}"}}]"#,
                user.first_name, user.last_name, user.is_verified
            ))
            .execute()
            .await
        {
            Ok(r) => {
                if r.status() == StatusCode::OK {
                    return Ok(user);
                }
                return Err("User not updated".to_string());
            }
            Err(_) => return Err("User not updated".to_string()),
        }
    }

    async fn delete(&mut self, phone_number: u64) -> Result<Option<User>, String> {
        match self
            .client
            .from("users")
            .eq("phone_number", phone_number.to_string())
            .delete()
            .execute()
            .await
        {
            Ok(r) => match r.text().await {
                Ok(t) => {
                    let body: Result<Vec<User>, serde_json::Error> = serde_json::from_str(&t);
                    match body {
                        Ok(users) => {
                            if users.len() == 0 {
                                return Err("User not deleted".to_string());
                            }
                            return Ok(Some(users[0].clone()));
                        }
                        Err(_) => Err("User not deleted".to_string()),
                    }
                }
                Err(_) => Err("User not deleted".to_string()),
            },
            Err(_) => return Err("User not deleted".to_string()),
        }
    }
}

fn unwrap_read_user(res: Result<Vec<User>, String>) -> Result<User, String> {
    match res {
        Ok(users) => {
            if users.len() == 0 {
                return Err("Expected len of users to be greater than 0".to_string());
            }
            return Ok(users[0].clone());
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create() {
        dotenv::dotenv().expect("dotenv to work");
        let supabase_url = std::env::var("SUPABASE_URL").expect("SUPABASE_URL must be set.");
        let supabase_api_key =
            std::env::var("SUPABASE_API_KEY").expect("SUPABASE_API_KEY must be set.");
        let mut user_repo = SupabaseRepo::new(&supabase_url, &supabase_api_key);

        user_repo
            .create(User {
                first_name: "Hunter".to_string(),
                last_name: "Simmons".to_string(),
                phone_number: 2028098681,
                is_verified: false,
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_read() {
        dotenv::dotenv().expect("dotenv to work");

        let supabase_url = std::env::var("SUPABASE_URL").expect("SUPABASE_URL must be set.");
        let supabase_api_key =
            std::env::var("SUPABASE_API_KEY").expect("SUPABASE_API_KEY must be set.");

        let user_repo = SupabaseRepo::new(&supabase_url, &supabase_api_key);

        let users = user_repo.read(2028098681).await.unwrap();
        assert_eq!(
            users[0],
            User {
                first_name: "Hunter".to_string(),
                last_name: "Simmons".to_string(),
                phone_number: 2028098681,
                is_verified: false,
            }
        )
    }

    #[tokio::test]
    async fn test_update() {
        dotenv::dotenv().expect("dotenv to work");
        let supabase_url = std::env::var("SUPABASE_URL").expect("SUPABASE_URL must be set.");
        let supabase_api_key =
            std::env::var("SUPABASE_API_KEY").expect("SUPABASE_API_KEY must be set.");
        let mut user_repo = SupabaseRepo::new(&supabase_url, &supabase_api_key);

        user_repo
            .update(User {
                first_name: "Hunter".to_string(),
                last_name: "Simmons".to_string(),
                phone_number: 2028098681,
                is_verified: true,
            })
            .await
            .unwrap();

        let users = user_repo.read(2028098681).await.unwrap();
        assert_eq!(
            users[0],
            User {
                first_name: "Hunter".to_string(),
                last_name: "Simmons".to_string(),
                phone_number: 2028098681,
                is_verified: true,
            }
        )
    }

    #[tokio::test]
    async fn test_delete() {
        dotenv::dotenv().expect("dotenv to work");
        let supabase_url = std::env::var("SUPABASE_URL").expect("SUPABASE_URL must be set.");
        let supabase_api_key =
            std::env::var("SUPABASE_API_KEY").expect("SUPABASE_API_KEY must be set.");
        let mut user_repo = SupabaseRepo::new(&supabase_url, &supabase_api_key);

        let deleted_user = user_repo.delete(2028098681).await.unwrap().unwrap();
        assert_eq!(
            deleted_user,
            User {
                first_name: "Hunter".to_string(),
                last_name: "Simmons".to_string(),
                phone_number: 2028098681,
                is_verified: false,
            }
        );
    }
}
