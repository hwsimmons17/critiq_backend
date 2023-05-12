use crate::{
    oauth::OAuth,
    places::search::DynPlacesSearch,
    repository::{places::DynPlacesRepo, user::DynUserRepo},
    sms::DynSMSVerify,
};

#[derive(Clone)]
pub struct AppState {
    pub user_repo: DynUserRepo,
    pub places_repo: DynPlacesRepo,
    pub sms_verify: DynSMSVerify,
    pub places_search: DynPlacesSearch,
    pub oauth: OAuth,
}
