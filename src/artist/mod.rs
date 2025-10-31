use std::{collections::HashSet, env, sync::Arc};

use dotenv::dotenv;
use futures::future;
use reqwest::{Client, header::CONTENT_TYPE};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::artist::{
    entities::Artist,
    responses::{Albums, ArtistsResponse, Tracks},
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
    pub async fn search_recursive(
        &self,
        name: String,
        n: usize,
        visited: Arc<Mutex<HashSet<String>>>,
    ) -> Vec<Artist> {
        {
            let mut visited_lock = visited.lock().await;
            if visited_lock.contains(&name) {
                return Vec::new();
            }
            visited_lock.insert(name.clone());
        }

        let mut total_artists = Vec::new();
        let artist = match self.search_artist(&name).await {
            Ok(artist) => artist,
            Err(_) => return Vec::new(),
        };
        let collabs = artist.collaborators.clone().unwrap_or_default();
        total_artists.push(artist);
        if n == 0 {
            return total_artists;
        }

        let futures = collabs.into_iter().map(|(name, _)| {
            let visited_clone = visited.clone();
            async move { self.search_recursive(name, n - 1, visited_clone).await }
        });
        let results = future::join_all(futures).await;
        for mut result in results {
            total_artists.append(&mut result);
        }

        total_artists
    }
}
