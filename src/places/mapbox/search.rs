use axum::async_trait;
use futures::future;
use reqwest::{StatusCode, Url};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{
    geo::Coordinates,
    places::{search::Search, Address, Place},
    repository::places::{PlacesRepository, ReadPlaceOptions},
};

pub struct MapboxSearchApi {
    access_token: String,
    foursquare_token: String,
    places_repo: Mutex<dyn PlacesRepository>,
}

#[derive(Deserialize, Serialize)]
struct MapboxPlace {
    mapbox_id: String,
    name: String,
    address: String,
    full_address: String,
    place_formatted: String,
    context: MapboxContext,
    external_ids: MapboxExternalIds,
}

#[derive(Deserialize, Serialize)]
struct MapboxContext {
    country: MapboxCountryContext,
    region: MapboxRegionContext,
    postcode: MapboxGenericContext,
    place: MapboxGenericContext,
    street: MapboxGenericContext,
}

#[derive(Deserialize, Serialize)]
struct MapboxCountryContext {
    name: String,
    country_code: String,
}

#[derive(Deserialize, Serialize)]
struct MapboxRegionContext {
    name: String,
    region_code: String,
}

#[derive(Deserialize, Serialize)]
struct MapboxGenericContext {
    name: String,
}

#[derive(Deserialize, Serialize)]
struct MapboxExternalIds {
    safegraph: Option<String>,
    foursquare: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct FoursquarePhoto {
    id: String,
    prefix: String,
    suffix: String,
}

impl MapboxPlace {
    fn convert_to_place(&self) -> Place {
        Place {
            id: 0,
            name: self.name.clone(),
            address: Address {
                address: self.address.clone(),
                full_address: Some(self.full_address.clone()),
                country: Some(self.context.country.name.clone()),
                region: Some(self.context.region.name.clone()),
                postcode: Some(self.context.postcode.name.clone()),
                place: Some(self.context.place.name.clone()),
                street: Some(self.context.street.name.clone()),
            },
            photos: None,
            website: None,
            foursquare_id: self.external_ids.foursquare.clone(),
        }
    }
}

#[async_trait]
impl Search for MapboxSearchApi {
    async fn search_for_place(
        &self,
        coordinates: Coordinates,
        search_string: String,
    ) -> Result<Vec<Place>, String> {
        let mut url = "https://api.mapbox.com/search/searchbox/v1/suggest"
            .parse::<Url>()
            .unwrap();
        url.query_pairs_mut()
            .append_pair("q", &search_string)
            .append_pair("access_token", &self.access_token)
            .append_pair("session_token", "[GENERATED-UUID]")
            .append_pair("language", "en")
            .append_pair(
                "proximity",
                &format!("{},{}", coordinates.longitude, coordinates.latitude),
            )
            .append_pair("poi_category", "food");

        let client = reqwest::Client::new();
        match client.get(url).send().await {
            Ok(res) => {
                if res.status() != StatusCode::OK {
                    return Err("Error requesting data from Mapbox".to_string());
                };
                let raw_body: String;
                match res.text().await {
                    Ok(b) => raw_body = b,
                    Err(_) => return Err("Error requesting data from Mapbox".to_string()),
                }
                let body: Result<Vec<MapboxPlace>, serde_json::Error> =
                    serde_json::from_str(&raw_body);
                let mapbox_places: Vec<MapboxPlace>;
                match body {
                    Ok(p) => mapbox_places = p,
                    Err(_) => return Err("Error requesting data from Mapbox".to_string()),
                }
                let places = future::try_join_all(mapbox_places.iter().map(|p| async move {
                    self.places_repo
                        .lock()
                        .await
                        .create(p.convert_to_place())
                        .await
                }))
                .await;

                return places;
            }
            Err(_) => Err("Error requesting data from Mapbox".to_string()),
        }
    }

    async fn get_photos(&self, place_id: u64) -> Result<Place, String> {
        let mut place: Place;
        match self
            .places_repo
            .lock()
            .await
            .read(ReadPlaceOptions {
                id: Some(place_id),
                name: None,
                address: None,
                postcode: None,
            })
            .await
        {
            Ok(places) => {
                if places.len() != 1 {
                    return Err("Error getting place".to_string());
                }
                place = places.first().unwrap().clone();
            }
            Err(_) => return Err("Error getting place".to_string()),
        };

        let foursquare_id: String;
        match place.foursquare_id.clone() {
            Some(id) => foursquare_id = id.clone(),
            None => return Ok(place),
        };

        let mut url = "https://api.foursquare.com/v3/places"
            .parse::<Url>()
            .unwrap();
        url.path_segments_mut()
            .map_err(|_| "cannot be base")
            .unwrap()
            .push(&foursquare_id);
        let client = reqwest::Client::new();
        let mut foursquare_photos: Vec<FoursquarePhoto>;
        match client
            .get(url)
            .header("Authorization:", self.foursquare_token.clone())
            .header("accept", "application/json")
            .send()
            .await
        {
            Ok(res) => {
                if res.status() != StatusCode::OK {
                    return Err("Error getting pictures".to_string());
                }
                match res.text().await {
                    Ok(t) => {
                        let body: Result<Vec<FoursquarePhoto>, serde_json::Error> =
                            serde_json::from_str(&t);
                        match body {
                            Ok(body) => foursquare_photos = body,
                            Err(_) => return Err("Error parsing JSON".to_string()),
                        }
                    }
                    Err(_) => return Err("Error parsing JSON".to_string()),
                }
            }
            Err(_) => return Err("Error getting pictures".to_string()),
        };

        let photos = foursquare_photos
            .iter_mut()
            .map(|photo| {
                photo.prefix.pop();
                return format!("{}{}", photo.prefix, photo.suffix);
            })
            .collect();
        place.photos = Some(photos);

        return self.places_repo.lock().await.update(place).await;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
