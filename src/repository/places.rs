use std::sync::Arc;

use axum::async_trait;
use tokio::sync::Mutex;

use crate::places::Place;

pub struct ReadPlaceOptions {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub address: Option<String>,
    pub postcode: Option<String>,
}

pub type DynPlacesRepo = Arc<Mutex<dyn PlacesRepository>>;

#[async_trait]
pub trait PlacesRepository: Send + Sync + 'static {
    async fn create(&mut self, place: &Place) -> Result<Place, String>;
    async fn read(&self, options: ReadPlaceOptions) -> Result<Vec<Place>, String>;
    async fn update(&mut self, place: Place) -> Result<Place, String>;
    async fn delete(&mut self, id: u64) -> Result<Option<Place>, String>;
}
