use std::env;

use dotenv::dotenv;
use reqwest::{
    IntoUrl, Url,
    blocking::Client,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artist {
    pub id: String,
    pub name: String,
}
#[derive(Serialize, Deserialize)]
pub struct ArtistsResponse {
    pub artists: Artists,
}
#[derive(Serialize, Deserialize)]
pub struct Artists {
    pub items: Vec<Artist>,
}

pub struct Music {
    client: Client,
    client_id: String,
    client_secret: String,
    access_token: String,
}

impl Music {
    pub fn new() -> Music {
        dotenv().unwrap();
        let client = Client::new();
        let client_id = env::var("CLIENT_ID").unwrap();
        let client_secret = env::var("CLIENT_SECRET").unwrap();
        let response: Value = client
            .post("https://accounts.spotify.com/api/token")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &client_id),
                ("client_secret", &client_secret),
            ])
            .send()
            .unwrap()
            .json()
            .unwrap();
        let access_token = response["access_token"].as_str().unwrap().to_string();

        Music {
            client,
            client_id,
            client_secret,
            access_token,
        }
    }
    fn get<T>(&self, req: impl IntoUrl) -> Result<T, reqwest::Error>
    where
        for<'a> T: Deserialize<'a>,
    {
        self.client
            .get(req)
            .header(AUTHORIZATION, format!("Bearer {}", self.access_token))
            .send()?
            .json()
    }
    pub fn search_artist(&self, query: &str) -> Result<Artists, reqwest::Error> {
        eprintln!("Finding {query}...");
        let artists: ArtistsResponse = self.get(
            Url::parse_with_params(
                "https://api.spotify.com/v1/search",
                [("q", query), ("type", "artist")],
            )
            .unwrap(),
        )?;
        eprintln!(
            "Found {} and {} others",
            artists.artists.items.first().unwrap().name,
            artists.artists.items.len() - 1
        );
        Ok(artists.artists)
    }
}
