use std::{env, time::Duration};

use dotenv::dotenv;
use futures::future;
use reqwest::{
    Client, IntoUrl, StatusCode,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::time::sleep;

use crate::music::{
    entities::Artist,
    responses::{Albums, Artists, Songs},
};

pub mod entities;
pub mod get_entities;
pub mod responses;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    Reqest(reqwest::Error),
    StatusCode(StatusCode),
}
impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqest(value)
    }
}
impl From<reqwest::StatusCode> for Error {
    fn from(value: reqwest::StatusCode) -> Self {
        Self::StatusCode(value)
    }
}

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
    async fn get<T: DeserializeOwned>(&self, req: impl IntoUrl + Clone) -> Result<T, Error>
where {
        loop {
            let resp = self
                .client
                .get(req.clone())
                .header(AUTHORIZATION, format!("Bearer {}", self.access_token))
                .send()
                .await?;
            match resp.status() {
                StatusCode::TOO_MANY_REQUESTS => {
                    let mut wait_secs = 1;
                    if let Some(wait) = resp.headers().get("Retry-after") {
                        if let Ok(secs) = wait.to_str().unwrap_or("1").parse::<u64>() {
                            wait_secs = secs;
                        }
                    }
                    if wait_secs > 5 {
                        eprintln!("Rate limited - waiting {}s...", wait_secs);
                    }
                    sleep(Duration::from_secs(wait_secs)).await;
                    continue;
                }
                StatusCode::OK => return resp.json().await.map_err(|e| e.into()),
                e => return Err(e.into()),
            }
        }
    }
    pub async fn search_recursive(&self, name: &str, n: usize) -> Vec<Artist> {
        let mut artists = Vec::new();

        let Ok(artist) = self.search_artist(name).await else {
            return artists;
        };

        artists.push(artist.clone());

        if n == 0 {
            return artists;
        }

        let Some(artists_collabs) = artist.collaborators else {
            return artists;
        };

        let futures = artists_collabs
            .iter()
            .map(|(collab_name, _)| Box::pin(self.search_recursive(&collab_name, n - 1)));
        let collabs: Vec<_> = future::join_all(futures)
            .await
            .into_iter()
            .flatten()
            .collect();

        artists.extend(collabs);
        // for (collab_name, _) in artists_collabs {
        //     let collabs = Box::pin(self.search_recursive(&collab_name, n - 1)).await;
        //     artists.extend(collabs);
        // }

        artists
    }
}
