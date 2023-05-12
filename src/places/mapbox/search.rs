use axum::async_trait;
use futures::future;
use reqwest::{StatusCode, Url};
use serde::{Deserialize, Serialize};

use crate::{
    geo::Coordinates,
    places::{search::Search, Address, Place},
    repository::places::{DynPlacesRepo, ReadPlaceOptions},
};

pub struct MapboxSearchApi {
    access_token: String,
    foursquare_token: String,
    places_repo: DynPlacesRepo,
}

#[derive(Deserialize, Serialize)]
struct MapboxSuggestions {
    suggestions: Vec<MapboxPlace>,
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
    country: Option<MapboxCountryContext>,
    region: Option<MapboxRegionContext>,
    postcode: Option<MapboxGenericContext>,
    place: Option<MapboxGenericContext>,
    street: Option<MapboxGenericContext>,
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
        let mut place = Place {
            id: 0,
            name: self.name.clone(),
            address: Address {
                address: self.address.clone(),
                full_address: Some(self.full_address.clone()),
                country: None,
                region: None,
                postcode: None,
                place: None,
                street: None,
            },
            photos: None,
            website: None,
            foursquare_id: self.external_ids.foursquare.clone(),
        };

        if let Some(country) = &self.context.country {
            place.address.country = Some(country.name.clone());
        };
        if let Some(region) = &self.context.region {
            place.address.region = Some(region.name.clone());
        };
        if let Some(postcode) = &self.context.postcode {
            place.address.postcode = Some(postcode.name.clone());
        };
        if let Some(p) = &self.context.place {
            place.address.place = Some(p.name.clone());
        };
        if let Some(street) = &self.context.street {
            place.address.street = Some(street.name.clone());
        };
        return place;
    }
}

impl MapboxSearchApi {
    pub fn new(access_token: &str, foursquare_token: &str, places_repo: DynPlacesRepo) -> Self {
        MapboxSearchApi {
            access_token: access_token.to_string(),
            foursquare_token: foursquare_token.to_string(),
            places_repo,
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
            .append_pair(
                "bbox",
                &format!(
                    "{},{},{},{}",
                    coordinates.longitude - 0.5,
                    coordinates.latitude - 0.5,
                    coordinates.longitude + 0.5,
                    coordinates.latitude + 0.5
                ),
            )
            .append_pair("types", "poi")
            .append_pair(
                "origin",
                &format!("{},{}", coordinates.longitude, coordinates.latitude),
            )
            .append_pair("poi_category", "food");

        let client = reqwest::Client::new();
        match client.get(url).send().await {
            Ok(res) => {
                if res.status() != StatusCode::OK {
                    eprintln!("Status code from Mapbox not 200, it was: {}", res.status());
                    eprintln!("Message for error was, {:?}", res.text().await);
                    return Err("Error requesting data from Mapbox".to_string());
                };
                let raw_body: String;
                match res.text().await {
                    Ok(b) => raw_body = b,
                    Err(e) => {
                        eprintln!("error getting text from Mapbox res: {}", e);
                        return Err("Error requesting data from Mapbox".to_string());
                    }
                }
                let body: Result<MapboxSuggestions, serde_json::Error> =
                    serde_json::from_str(&raw_body);
                let mapbox_places: Vec<MapboxPlace>;
                match body {
                    Ok(p) => mapbox_places = p.suggestions,
                    Err(e) => {
                        eprintln!(
                            "error unmarshalling response from Mapbox: {}. RawBody was {}",
                            e, raw_body
                        );
                        return Err("Error requesting data from Mapbox".to_string());
                    }
                }
                let places = future::try_join_all(mapbox_places.iter().map(|p| async move {
                    self.places_repo
                        .lock()
                        .await
                        .create(&p.convert_to_place())
                        .await
                }))
                .await;

                return places;
            }
            Err(e) => {
                eprintln!("error sending response to Mapbox: {}", e);
                Err("Error requesting data from Mapbox".to_string())
            }
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
            .push(&foursquare_id)
            .push("photos");
        let client = reqwest::Client::new();
        let mut foursquare_photos: Vec<FoursquarePhoto>;
        match client
            .get(url)
            .header("Authorization", self.foursquare_token.clone())
            .header("accept", "application/json")
            .send()
            .await
        {
            Ok(res) => {
                if res.status() != StatusCode::OK {
                    eprintln!("Status code: {}, when getting pictures", res.status());
                    return Err("Error getting pictures".to_string());
                }
                match res.text().await {
                    Ok(t) => {
                        let body: Result<Vec<FoursquarePhoto>, serde_json::Error> =
                            serde_json::from_str(&t);
                        match body {
                            Ok(body) => foursquare_photos = body,
                            Err(_) => {
                                eprintln!("Error parsing JSON from foursquare, {}", t);
                                return Err("Error parsing JSON".to_string());
                            }
                        }
                    }
                    Err(_) => return Err("Error parsing JSON".to_string()),
                }
            }
            Err(e) => {
                eprintln!("error sending to foursquare: {}", e);
                return Err("Error getting pictures".to_string());
            }
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
