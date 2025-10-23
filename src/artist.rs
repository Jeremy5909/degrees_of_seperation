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

pub struct Music {
    client: Client,
}
impl Music {
    pub fn new() -> Music {
        Music {
            client: Client::new(),
        }
    }
    pub fn search(&self, query: &str) -> Result<Artists, reqwest::Error> {
        let url = Url::parse_with_params(
            "https://www.musicbrainz.org/ws/2/artist",
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
