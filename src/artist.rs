use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Artists {
    pub artists: Vec<ArtistWithId>,
}

#[derive(Serialize, Deserialize)]
struct ArtistWithId {
    id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Artist {
    pub name: String,
}

pub fn search_artist(client: &Client, name: &str) -> Result<(String, Artist), String> {
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

    let artist = artists
        .artists
        .into_iter()
        .next()
        .ok_or("Artist not found")?;
    Ok((artist.id, Artist { name: artist.name }))
}
