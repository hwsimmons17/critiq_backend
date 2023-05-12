use axum::async_trait;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    places::{Address, Place},
    repository::places::{PlacesRepository, ReadPlaceOptions},
};

use super::SupabaseRepo;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RepoPlace {
    pub id: u64,
    pub name: String,
    pub address: String,
    pub full_address: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub postcode: Option<String>,
    pub place: Option<String>,
    pub street: Option<String>,
    pub photos: Option<Vec<String>>,
    pub website: Option<String>,
    pub foursquare_id: Option<String>,
}

impl RepoPlace {
    fn convert_to_place(&self) -> Place {
        Place {
            id: self.id,
            name: self.name.clone(),
            address: Address {
                address: self.address.clone(),
                full_address: self.full_address.clone(),
                country: self.country.clone(),
                region: self.region.clone(),
                postcode: self.postcode.clone(),
                place: self.place.clone(),
                street: self.street.clone(),
            },
            photos: self.photos.clone(),
            website: self.website.clone(),
            foursquare_id: self.foursquare_id.clone(),
        }
    }
}

#[async_trait]
impl PlacesRepository for SupabaseRepo {
    async fn create(&mut self, place: &Place) -> Result<Place, String> {
        match self
            .client
            .from("places")
            .insert(format_create_command(&place))
            .execute()
            .await
        {
            Ok(r) => {
                if r.status() == StatusCode::CREATED {
                    return parse_places(r.text().await);
                }
                if r.status() == StatusCode::CONFLICT {
                    return unwrap_read_places(
                        self.read(ReadPlaceOptions {
                            id: None,
                            name: None,
                            address: Some(place.address.address.clone()),
                            postcode: None,
                        })
                        .await,
                    );
                }

                eprintln!(
                    "Status code not what was expected when creating place: {}",
                    r.status()
                );
                return Err("Place not created".to_string());
            }
            Err(_) => return Err("User not created".to_string()),
        }
    }
    async fn read(&self, options: ReadPlaceOptions) -> Result<Vec<Place>, String> {
        let mut client = self.client.from("places");
        if let Some(id) = options.id {
            client = client.eq("id", id.to_string())
        };
        if let Some(address) = options.address {
            client = client.eq("address", address)
        };
        if let Some(name) = options.name {
            client = client.eq("name", name)
        };
        if let Some(postcode) = options.postcode {
            client = client.eq("postcode", postcode)
        };

        match client.select("*").execute().await {
            Ok(r) => match r.text().await {
                Ok(t) => {
                    let body: Result<Vec<RepoPlace>, serde_json::Error> = serde_json::from_str(&t);
                    match body {
                        Ok(b) => Ok(b.iter().map(|p| p.convert_to_place()).collect()),
                        Err(_) => Err("Could not read places".to_string()),
                    }
                }
                Err(_) => Err("Could not read places".to_string()),
            },
            Err(_) => return Err("Could not read places".to_string()),
        }
    }
    async fn update(&mut self, place: Place) -> Result<Place, String> {
        let return_place = place.clone();
        match self
            .client
            .from("places")
            .eq("id", place.id.to_string())
            .update(format_create_command(&place))
            .execute()
            .await
        {
            Ok(r) => {
                if r.status() == StatusCode::OK {
                    return Ok(return_place);
                }
                eprintln!(
                    "Expected status to be 200 when updating place, got: {}",
                    r.status()
                );
                eprintln!("{:?}", r.text().await);
                eprintln!("{}", format_create_command(&place));
                return Err("Place not updated".to_string());
            }
            Err(_) => return Err("Place not updated".to_string()),
        }
    }
    async fn delete(&mut self, id: u64) -> Result<Option<Place>, String> {
        match self
            .client
            .from("places")
            .eq("id", id.to_string())
            .delete()
            .execute()
            .await
        {
            Ok(r) => match r.text().await {
                Ok(t) => {
                    let body: Result<Vec<RepoPlace>, serde_json::Error> = serde_json::from_str(&t);
                    match body {
                        Ok(places) => {
                            if places.len() == 0 {
                                return Err("Place not deleted".to_string());
                            }
                            return Ok(Some(places[0].clone().convert_to_place()));
                        }
                        Err(_) => Err("Place not deleted".to_string()),
                    }
                }
                Err(_) => Err("Place not deleted".to_string()),
            },
            Err(_) => return Err("Place not deleted".to_string()),
        }
    }
}

fn format_create_command(place: &Place) -> String {
    let p = place.clone();
    let mut beginning = format!(
        r#"[{{"name": "{}", "address": "{}"#,
        place.name, place.address.address
    );
    if let Some(full_address) = p.address.full_address {
        beginning = beginning + &format!(r#"", "full_address": "{}"#, full_address)
    }
    if let Some(country) = p.address.country {
        beginning = beginning + &format!(r#"", "country": "{}"#, country)
    }
    if let Some(region) = p.address.region {
        beginning = beginning + &format!(r#"", "region": "{}"#, region)
    }
    if let Some(postcode) = p.address.postcode {
        beginning = beginning + &format!(r#"", "postcode": "{}"#, postcode)
    }
    if let Some(place) = p.address.place {
        beginning = beginning + &format!(r#"", "place": "{}"#, place)
    }
    if let Some(street) = p.address.street {
        beginning = beginning + &format!(r#"", "street": "{}"#, street)
    }
    if let Some(website) = p.website {
        beginning = beginning + &format!(r#"", "website": "{}"#, website)
    }
    if let Some(foursquare_id) = p.foursquare_id {
        beginning = beginning + &format!(r#"", "foursquareId": "{}"#, foursquare_id)
    }
    if let Some(photos) = p.photos.clone() {
        beginning = beginning + &format!(r#"", "photos": {:?}"#, photos)
    }
    let end: &str;
    match p.photos {
        Some(_) => end = r#"}]"#,
        None => end = r#""}]"#,
    };
    return beginning + &end;
}

fn parse_places(res: Result<String, reqwest::Error>) -> Result<Place, String> {
    match res {
        Ok(r) => {
            let body: Result<Vec<RepoPlace>, serde_json::Error> = serde_json::from_str(&r);
            return unwrap_read_places_json(body);
        }
        Err(_) => return Err("Error with request".to_string()),
    }
}

pub fn unwrap_read_places(res: Result<Vec<Place>, String>) -> Result<Place, String> {
    match res {
        Ok(places) => {
            if places.len() == 0 {
                return Err("Expected len of places to be greater than 0".to_string());
            }
            return Ok(places[0].clone());
        }
        Err(e) => Err(e),
    }
}

pub fn unwrap_read_places_json(
    res: Result<Vec<RepoPlace>, serde_json::Error>,
) -> Result<Place, String> {
    match res {
        Ok(places) => {
            if places.len() == 0 {
                return Err("Expected len of places to be greater than 0".to_string());
            };
            return Ok(places[0].clone().convert_to_place());
        }
        Err(e) => Err("Error unmarshaling JSON".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use crate::places::Address;

    use super::*;

    #[tokio::test]
    async fn test_create() {
        dotenv::dotenv().expect("dotenv to work");
        let supabase_url = std::env::var("SUPABASE_URL").expect("SUPABASE_URL must be set.");
        let supabase_api_key =
            std::env::var("SUPABASE_API_KEY").expect("SUPABASE_API_KEY must be set.");
        let mut repo = SupabaseRepo::new(&supabase_url, &supabase_api_key);

        repo.create(&Place {
            id: 0,
            name: "Arlo".to_string(),
            address: Address {
                address: "123 N Fake Street".to_string(),
                full_address: Some("123 N Fake Street".to_string()),
                country: Some("USA".to_string()),
                region: Some("Utah".to_string()),
                postcode: Some("84106".to_string()),
                place: Some("Salt Lake City".to_string()),
                street: Some("N Fake Street".to_string()),
            },
            photos: None,
            website: None,
            foursquare_id: None,
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_read() {
        dotenv::dotenv().expect("dotenv to work");

        let supabase_url = std::env::var("SUPABASE_URL").expect("SUPABASE_URL must be set.");
        let supabase_api_key =
            std::env::var("SUPABASE_API_KEY").expect("SUPABASE_API_KEY must be set.");

        let repo = SupabaseRepo::new(&supabase_url, &supabase_api_key);

        let places = repo
            .read(ReadPlaceOptions {
                id: None,
                name: Some("Arlo".to_string()),
                address: None,
                postcode: None,
            })
            .await
            .unwrap();
        assert_eq!(places[0].name, "Arlo".to_string())
    }

    #[tokio::test]
    async fn test_update() {
        dotenv::dotenv().expect("dotenv to work");
        let supabase_url = std::env::var("SUPABASE_URL").expect("SUPABASE_URL must be set.");
        let supabase_api_key =
            std::env::var("SUPABASE_API_KEY").expect("SUPABASE_API_KEY must be set.");
        let mut repo = SupabaseRepo::new(&supabase_url, &supabase_api_key);

        repo.update(Place {
            id: 10,
            name: "Arlo".to_string(),
            address: Address {
                address: "123 N Another Fake Street".to_string(),
                full_address: Some("123 N Another Fake Street".to_string()),
                country: Some("USA".to_string()),
                region: Some("Utah".to_string()),
                postcode: Some("84106".to_string()),
                place: Some("Salt Lake City".to_string()),
                street: Some("N Another Fake Street".to_string()),
            },
            photos: None,
            website: None,
            foursquare_id: None,
        })
        .await
        .unwrap();

        let places = repo
            .read(ReadPlaceOptions {
                id: Some(10),
                name: None,
                address: None,
                postcode: None,
            })
            .await
            .unwrap();
        assert_eq!(places[0].address.address, "123 N Another Fake Street")
    }
}
