use std::{default, env};

use dotenv::dotenv;
use reqwest::{Client, header::CONTENT_TYPE};
use serde_json::Value;

use crate::artist::{
    entities::Artist,
    responses::{Albums, Artists, Songs},
};

pub mod entities;
pub mod get_functions;
pub mod responses;

#[allow(dead_code)]
pub struct Music {
    client: Client,
    client_id: String,
    client_secret: String,
    access_token: String,
}
impl Music {
    pub async fn new() -> Music {
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
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let access_token = response["access_token"].as_str().unwrap().to_string();

        Music {
            client,
            client_id,
            client_secret,
            access_token,
        }
    }
    pub async fn search_recursive(&self, name: &str, n: usize) -> Vec<Artist> {
        let mut artists = Vec::new();

        let artist = self.search_artist(name).await.unwrap();
        artists.push(artist.clone());

        if n == 0 {
            return artists;
        }

        let Some(artists_collabs) = artist.collaborators else {
            return artists;
        };
        for (collab_name, _) in artists_collabs {
            let collabs = Box::pin(self.search_recursive(&collab_name, n - 1)).await;
            artists.extend(collabs);
        }

        artists
    }
}
