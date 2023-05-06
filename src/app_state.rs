use crate::repository::user::UserRepository;

#[derive(Clone)]
pub struct AppState<U: UserRepository> {
    pub user_repo: U,
}
