use std::collections::VecDeque;

use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct Artists {
    artists: VecDeque<Artist>,
}

#[derive(Debug, Deserialize)]
struct Artist {
    id: String,
    name: String,
}

fn search_artist(client: &Client, name: &str) -> Result<Artist, String> {
    let url = format!("https://musicbrainz.org/ws/2/artist/?query=artist:{name}&fmt=json");
    let result = client
        .get(url)
        .header(
            "User-Agent",
            "degrees-of-seperation/0.1 (jeremyherczeg@gmail.com)",
        )
        .send()
        .map_err(|e| e.to_string())?;
    let mut search: Artists = result.json().map_err(|e| e.to_string())?;
    search.artists.pop_front().ok_or("Artist not found".into())
}

fn main() {
    let client = Client::new();
    let artist = search_artist(&client, "playboi carti");
    println!("{:#?}", artist);
}
