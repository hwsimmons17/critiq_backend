use critiq_backend::{repository::local::user::LocalUserRepository, run};

#[tokio::main]
async fn main() {
    let user_repo = LocalUserRepository::new();
    run(user_repo).await
}
