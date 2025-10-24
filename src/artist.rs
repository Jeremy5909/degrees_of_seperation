use reqwest::{IntoUrl, Url, blocking::Client};
use serde::{Deserialize, Serialize};

pub struct Music {
    client: Client,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub songs: Option<Vec<Song>>,
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
pub struct Recordings {
    pub recordings: Vec<Song>,
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
        let mut all_songs = Vec::new();
        let mut offset = 0;
        let mut page = 0;
        let limit = 100;

        loop {
            let songs = self.get(
                Url::parse_with_params(
                    &format!(
                        "https://www.musicbrainz.org/ws/2/recording?artist={}",
                        artist.id
                    ),
                    [
                        ("fmt", "json"),
                        ("limit", &limit.to_string()),
                        ("offset", &offset.to_string()),
                    ],
                )
                .unwrap(),
            );
            if songs.is_err() {
                eprintln!("uhh");
                continue;
            }
            let songs: Recordings = songs.unwrap();
            let release_count = songs.recordings.len();
            all_songs.extend(songs.recordings);
            offset += release_count;
            page += 1;
            eprintln!("Going to page {page}");

            if release_count < limit {
                break;
            }
        }
        artist.songs = Some(all_songs);
        eprintln!(
            "Found {} {} songs",
            artist.songs.clone().unwrap().len(),
            artist.name
        );
    }
}
