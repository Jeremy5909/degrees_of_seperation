use std::collections::HashMap;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Artists {
    pub artists: Vec<Artist>,
}
impl Artists {
    pub fn into_hashmap(self) -> HashMap<String, Artist> {
        let mut hashmap = HashMap::new();
        for artist in self.artists {
            hashmap.insert(artist.id.clone(), artist);
        }
        hashmap
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Artist {
    #[serde(skip_serializing)]
    pub id: String,
    pub name: String,
}

pub fn search_artist(client: &Client, name: &str) -> Result<Artist, String> {
    let url = format!("https://musicbrainz.org/ws/2/artist/?query=artist:{name}&fmt=json");
    let artists: Artists = client
        .get(url)
        .header(
            "User-Agent",
            "degrees-of-seperation/0.1 (jeremyherczeg@gmail.com)",
        )
        .send()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    artists
        .artists
        .into_iter()
        .next()
        .ok_or("Artist not found".into())
}
