use axum::{extract::State, http::StatusCode, Json};
use futures::future;
use serde::{Deserialize, Serialize};

use crate::{app_state::AppState, geo::Coordinates, places::Place};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    place_name: String,
    location: Coordinates,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    places: Vec<Place>,
}

#[axum_macros::debug_handler]
pub async fn search_for_place(
    State(app_state): State<AppState>,
    Json(payload): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, String)> {
    let places_repo = &app_state.places_repo;
    let places_search = &app_state.places_search;

    let places: Vec<Place>;
    match places_search
        .lock()
        .await
        .search_for_place(payload.location, payload.place_name)
        .await
    {
        Ok(p) => places = p,
        Err(e) => {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, e));
        }
    };
    let return_places = places.clone();

    let places_repo = places_repo.clone();
    let places_search = places_search.clone();
    tokio::spawn(async move {
        let places_repo = &places_repo;
        let places_search = &places_search;

        let results = future::try_join_all(places.iter().map(|place| async move {
            if place.photos == None {
                return places_search.lock().await.get_photos(place.id).await;
            }
            return Ok(place.clone());
        }))
        .await;

        match results {
            Ok(p) => {
                return future::try_join_all(
                    p.iter()
                        .map(|place| async move { places_repo.lock().await.create(place).await }),
                )
                .await;
            }
            Err(e) => Err(e),
        }
    });

    return Ok(axum::Json(SearchResponse {
        places: return_places,
    }));
}
