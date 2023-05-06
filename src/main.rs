use critiq_backend::{app_state::AppState, repository::local::user::LocalUserRepository, run};

#[tokio::main]
async fn main() {
    let user_repo = LocalUserRepository::new();
    let app_state = AppState::<LocalUserRepository> { user_repo };
    run(app_state).await
}
