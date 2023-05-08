use critiq_backend::{app_state::AppState, repository::local::user::LocalUserRepository, run};

#[tokio::main]
async fn main() {
    let user_repo = LocalUserRepository::new();
    run(user_repo).await
}
