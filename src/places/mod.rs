pub mod mapbox;
pub mod search;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    pub id: u64,
    pub name: String,
    pub address: Address,
    pub photos: Option<Vec<String>>,
    pub website: Option<String>,
    pub foursquare_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub address: String,
    pub full_address: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub postcode: Option<String>,
    pub place: Option<String>,
    pub street: Option<String>,
}
