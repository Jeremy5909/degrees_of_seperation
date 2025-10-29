use std::{collections::HashMap, env, fmt::Display, vec::IntoIter};

use dotenv::dotenv;
use reqwest::{
    IntoUrl, Url,
    blocking::Client,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

pub type ArtistSmall = HashMap<String, String>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artist {
    pub id: String,
    pub name: String,
    collaborators: Option<ArtistSmall>,
}

#[derive(Serialize, Deserialize)]
struct ArtistsResponse {
    pub artists: Artists,
}
impl IntoIterator for ArtistsResponse {
    type Item = Artist;

    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.artists.items.into_iter()
    }
}
#[derive(Serialize, Deserialize)]
pub struct Artists {
    pub items: Vec<Artist>,
}

#[derive(Deserialize)]
struct Albums {
    items: Vec<Album>,
}
impl IntoIterator for Albums {
    type Item = Album;

    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}
#[derive(Deserialize, Debug, Clone, Serialize)]
struct Album {
    name: String,
    id: String,
    tracks: Option<Vec<Track>>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
struct Tracks {
    items: Vec<Track>,
}
impl IntoIterator for Tracks {
    type Item = Track;

    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

#[derive(Deserialize, Debug, Clone, Serialize)]
struct Track {
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
    fn get_entities<T: IntoIterator + DeserializeOwned>(
        &self,
        lhs: Entity,
        id: &str,
        rhs: Entity,
        params: Option<Vec<(&str, &str)>>,
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
            if let Some(extra_params) = &params {
                all_params.extend(extra_params);
            }
            let fetched_entities: T = self
                .get(
                    Url::parse_with_params(
                        &format!("https://api.spotify.com/v1/{lhs}/{id}/{rhs}"),
                        all_params,
                    )
                    .unwrap(),
                )
                .unwrap();
            let fetched_entities: Vec<_> = fetched_entities.into_iter().collect();
            let length = fetched_entities.len();
            eprintln!("got {}", length);
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
            println!("going to page {page}");
        }
        println!("done fetching all entities");
        Ok(entities)
    }
    pub fn search_artist(&self, query: &str) -> Result<Artist, reqwest::Error> {
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
        let artists = artists.artists.items;
        let mut artist = artists.into_iter().next().unwrap();

        artist.collaborators = Some(
            self.get_artist_collaborators(&mut artist)
                .into_iter()
                .collect(),
        );

        Ok(artist)
    }
    fn get_artist_collaborators(&self, artist: &mut Artist) -> ArtistSmall {
        let tracks = self.get_artist_tracks(artist);
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
    fn get_artist_tracks(&self, artist: &Artist) -> Vec<Track> {
        let albums = self.get_artist_albums(artist);
        let mut total_tracks = Vec::new();
        for album in albums {
            total_tracks.extend(self.get_album_tracks(&album));
        }
        total_tracks
    }
    fn get_artist_albums(&self, artist: &Artist) -> Vec<Album> {
        eprintln!("Finding {}'s albums...", artist.name);
        let albums: Vec<Album> = self
            .get_entities::<Albums>(
                Entity::Artists,
                &artist.id,
                Entity::Albums,
                Some(vec![("include_groups", "album,single")]),
                None,
            )
            .unwrap();
        eprintln!("Found {} albums", albums.len());

        albums
    }
    fn get_album_tracks(&self, album: &Album) -> Vec<Track> {
        eprintln!("Finding {}'s songs...", album.name);
        let tracks: Vec<Track> = self
            .get_entities::<Tracks>(Entity::Albums, &album.id, Entity::Tracks, None, None)
            .unwrap();
        eprintln!("Found {} songs", tracks.len());
        tracks
    }
}
