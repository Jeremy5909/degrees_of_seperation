use reqwest::{IntoUrl, Url, blocking::Client};
use serde::{Deserialize, Serialize};

pub struct Music {
    client: Client,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub songs: Option<Songs>,
}
#[derive(Serialize, Deserialize)]
pub struct Artists {
    pub artists: Vec<Artist>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Song {
    pub id: String,
    pub title: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Songs {
    pub releases: Vec<Song>,
}

impl Music {
    pub fn new() -> Music {
        Music {
            client: Client::new(),
        }
    }
    fn get<T>(&self, req: impl IntoUrl) -> Result<T, reqwest::Error>
    where
        for<'a> T: Deserialize<'a>,
    {
        self.client
            .get(req)
            .header(
                "User-Agent",
                "degrees-of-seperation/0.1 (jeremyherczeg@gmail.com)",
            )
            .send()?
            .json()
    }
    pub fn search(&self, query: &str) -> Result<Artists, reqwest::Error> {
        eprintln!("Finding {query}...");
        let artists: Artists = self.get(
            Url::parse_with_params(
                "https://www.musicbrainz.org/ws/2/artist",
                [("query", query), ("fmt", "json")],
            )
            .unwrap(),
        )?;
        eprintln!(
            "Found {} and {} others",
            artists.artists.first().unwrap().name,
            artists.artists.len() - 1
        );
        Ok(artists)
    }
    pub fn fetch_songs(&self, artist: &mut Artist) {
        eprintln!("Fetching {}'s songs...", artist.name);
        let songs = self
            .get(
                Url::parse_with_params(
                    &format!(
                        "https://www.musicbrainz.org/ws/2/release?artist={}",
                        artist.id
                    ),
                    [("fmt", "json")],
                )
                .unwrap(),
            )
            .unwrap();
        artist.songs = Some(songs);
        eprintln!(
            "Found {} {} songs",
            artist.songs.clone().unwrap().releases.len(),
            artist.name
        );
    }
}
