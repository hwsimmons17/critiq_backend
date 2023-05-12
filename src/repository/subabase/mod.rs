pub mod places;
pub mod user;

pub struct SupabaseRepo {
    client: postgrest::Postgrest,
}

impl SupabaseRepo {
    pub fn new(url: &str, api_key: &str) -> Self {
        let client = postgrest::Postgrest::new(url).insert_header("apikey", api_key);
        SupabaseRepo { client }
    }
}
