use std::time::Duration;

use futures::future;
use reqwest::{IntoUrl, StatusCode, Url, header::AUTHORIZATION};
use serde::de::DeserializeOwned;
use tokio::time::sleep;

use crate::music::{
    self, Albums, Artists, Music, Songs,
    entities::{Album, Artist, ArtistSmall, Entity, Song},
};

impl Music {
    async fn get<T: DeserializeOwned>(&self, req: impl IntoUrl + Clone) -> Result<T, music::Error>
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
                    println!("Rate limited - waiting {}s...", wait_secs);
                    sleep(Duration::from_secs(wait_secs)).await;
                    continue;
                }
                StatusCode::OK => return resp.json().await.map_err(|e| e.into()),
                e => return Err(e.into()),
            }
        }
    }
    async fn get_entities<T: IntoIterator + DeserializeOwned>(
        &self,
        lhs: Entity,
        id: &str,
        rhs: Entity,
        params: Vec<(&str, &str)>,
        number_pages: Option<usize>,
    ) -> Result<Vec<T::Item>, music::Error> {
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
    pub async fn search_artist(&self, query: &str) -> Result<Artist, music::Error> {
        let artists: Artists = self
            .get(
                Url::parse_with_params(
                    "https://api.spotify.com/v1/search",
                    [("q", query), ("type", "artist")],
                )
                .unwrap(),
            )
            .await?;
        let artists: Vec<_> = artists.into_iter().collect();
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
        let tracks = self.get_artist_songs(artist).await;
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
    async fn get_artist_songs(&self, artist: &Artist) -> Vec<Song> {
        let albums = self.get_artist_albums(artist).await;

        let results =
            future::join_all(albums.iter().map(|album| self.get_album_songs(&album))).await;

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
    async fn get_album_songs(&self, album: &Album) -> Vec<Song> {
        self.get_entities::<Songs>(Entity::Albums, &album.id, Entity::Songs, vec![], None)
            .await
            .unwrap_or_default()
    }
}
