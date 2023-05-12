use axum::async_trait;

use crate::geo::Coordinates;

use super::Place;

#[async_trait]
pub trait Search {
    async fn search_for_place(
        &self,
        coordinates: Coordinates,
        search_string: String,
    ) -> Result<Vec<Place>, String>;

    async fn get_photos(&self, place_id: u64) -> Result<Place, String>;
}
