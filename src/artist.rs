use std::fmt::Display;

use reqwest::{Url, blocking::Client};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Artists {
    pub artists: Vec<Artist>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
}

#[derive(Debug)]
pub enum Entity {
    Artist,
    Release,
    Recording,
}
impl Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

pub struct Music {
    client: Client,
}
impl Music {
    pub fn new() -> Music {
        Music {
            client: Client::new(),
        }
    }
    pub fn search(&self, entity_type: Entity, query: &str) -> Result<Artists, reqwest::Error> {
        let url = Url::parse_with_params(
            &format!("https://www.musicbrainz.org/ws/2/{entity_type}"),
            [("query", query), ("fmt", "json")],
        )
        .unwrap();

        self.client
            .get(url)
            .header(
                "User-Agent",
                "degrees-of-seperation/0.1 (jeremyherczeg@gmail.com)",
            )
            .send()?
            .json()
    }
}
