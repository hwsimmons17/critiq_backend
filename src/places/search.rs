use std::sync::Arc;

use axum::async_trait;
use tokio::sync::Mutex;

use crate::geo::Coordinates;

use super::Place;

pub type DynPlacesSearch = Arc<Mutex<dyn Search>>;

#[async_trait]
pub trait Search: Send + Sync + 'static {
    async fn search_for_place(
        &self,
        coordinates: Coordinates,
        search_string: String,
    ) -> Result<Vec<Place>, String>;

    async fn get_photos(&self, place_id: u64) -> Result<Place, String>;
}
