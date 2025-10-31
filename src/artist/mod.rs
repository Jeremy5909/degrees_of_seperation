use std::{collections::HashMap, env, fmt::Display};

use dotenv::dotenv;
use futures::future;
use reqwest::{
    Client, IntoUrl, Url,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::artist::responses::{Albums, ArtistsResponse, Tracks};

mod responses;

pub type ArtistSmall = HashMap<String, String>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub collaborators: Option<ArtistSmall>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Album {
    name: String,
    id: String,
    tracks: Option<Vec<Track>>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Track {
    name: String,
    id: String,
    artists: Vec<Artist>,
}

#[derive(Debug)]
enum Entity {
    Albums,
    Artists,
    Tracks,
}
impl Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lower = format!("{:?}", self).to_lowercase();
        write!(f, "{lower}")
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
    async fn get<T: DeserializeOwned>(&self, req: impl IntoUrl) -> Result<T, reqwest::Error>
where {
        self.client
            .get(req)
            .header(AUTHORIZATION, format!("Bearer {}", self.access_token))
            .send()
            .await?
            .json()
            .await
    }
    async fn get_entities<T: IntoIterator + DeserializeOwned>(
        &self,
        lhs: Entity,
        id: &str,
        rhs: Entity,
        params: Vec<(&str, &str)>,
        number_pages: Option<usize>,
    ) -> Result<Vec<T::Item>, reqwest::Error> {
        let mut page = 0;
        let limit = 50;

        let mut entities: Vec<T::Item> = Vec::new();

        loop {
            let offset_str = (page * limit).to_string();
            let limit_str = limit.to_string();
            let mut all_params = vec![
                ("limit", limit_str.as_str()),
                ("offset", offset_str.as_str()),
            ];

            all_params.extend(params.clone());

            let fetched_entities: T = self
                .get(
                    Url::parse_with_params(
                        &format!("https://api.spotify.com/v1/{lhs}/{id}/{rhs}"),
                        all_params,
                    )
                    .unwrap(),
                )
                .await?;
            let fetched_entities: Vec<_> = fetched_entities.into_iter().collect();
            let length = fetched_entities.len();
            eprintln!("Got {length} {rhs} from {lhs}");
            entities.extend(fetched_entities);
            if length < limit {
                break;
            }
            if let Some(number_pages) = number_pages {
                if page > number_pages {
                    break;
                }
            }

            page += 1;
        }
        Ok(entities)
    }
    pub async fn search_artist(&self, query: &str) -> Result<Artist, reqwest::Error> {
        let artists: ArtistsResponse = self
            .get(
                Url::parse_with_params(
                    "https://api.spotify.com/v1/search",
                    [("q", query), ("type", "artist")],
                )
                .unwrap(),
            )
            .await?;
        let artists = artists.artists.items;
        let mut artist = artists.into_iter().next().unwrap();

        artist.collaborators = Some(
            self.get_artist_collaborators(&mut artist)
                .await
                .into_iter()
                .collect(),
        );

        Ok(artist)
    }
    pub async fn get_artist_collaborators(&self, artist: &mut Artist) -> ArtistSmall {
        let tracks = self.get_artist_tracks(artist).await;
        let mut collaborators = ArtistSmall::new();
        for track in tracks {
            collaborators.extend(
                track
                    .artists
                    .into_iter()
                    .map(|artist| (artist.name, artist.id)),
            );
        }
        eprintln!("Found {} collaborators", collaborators.len());
        collaborators
    }
    async fn get_artist_tracks(&self, artist: &Artist) -> Vec<Track> {
        let albums = self.get_artist_albums(artist).await;

        let futures: Vec<_> = albums
            .iter()
            .map(|album| self.get_album_tracks(&album))
            .collect();
        let results = future::join_all(futures).await;

        results.into_iter().flatten().collect()
    }
    async fn get_artist_albums(&self, artist: &Artist) -> Vec<Album> {
        eprintln!("Finding {}'s albums...", artist.name);
        let albums: Vec<Album> = self
            .get_entities::<Albums>(
                Entity::Artists,
                &artist.id,
                Entity::Albums,
                vec![("include_groups", "album,single")],
                None,
            )
            .await
            .unwrap_or_default();
        eprintln!("Found {} albums", albums.len());

        albums
    }
    async fn get_album_tracks(&self, album: &Album) -> Vec<Track> {
        self.get_entities::<Tracks>(Entity::Albums, &album.id, Entity::Tracks, vec![], None)
            .await
            .unwrap_or_default()
    }
}
