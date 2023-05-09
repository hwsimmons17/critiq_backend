use crate::{oauth::OAuth, repository::user::DynUserRepo, sms::DynSMSVerify};

#[derive(Clone)]
pub struct AppState {
    pub user_repo: DynUserRepo,
    pub sms_verify: DynSMSVerify,
    pub oauth: OAuth,
}
