use futures::future;
use reqwest::Url;
use serde::de::DeserializeOwned;

use crate::music::{
    self, Albums, Artists, Music, Songs,
    entities::{Album, Artist, ArtistSmall, Entity, Song},
};

impl Music {
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
        let mut artist = artists.into_iter().next().unwrap();

        artist.collaborators = Some(
            self.get_all_collabs(&mut artist)
                .await
                .into_iter()
                .collect(),
        );

        Ok(artist)
    }
    pub async fn get_all_collabs(&self, artist: &mut Artist) -> ArtistSmall {
        let all_songs = self.get_all_songs(artist).await;

        let collaborators: ArtistSmall = all_songs
            .into_iter()
            .flat_map(|song| {
                song.artists
                    .into_iter()
                    .map(|artist| (artist.name, artist.id))
            })
            .collect();

        eprintln!(
            "Found {} {} collaborators",
            collaborators.len(),
            artist.name
        );
        collaborators
    }
    pub async fn get_all_songs(&self, artist: &Artist) -> Vec<Song> {
        let albums = self.get_all_albums(artist).await;

        let results = future::join_all(albums.iter().map(async |album| {
            self.get_entities::<Songs>(Entity::Albums, &album.id, Entity::Songs, vec![], None)
                .await
                .unwrap_or_default()
        }))
        .await;

        results.into_iter().flatten().collect()
    }
    pub async fn get_all_albums(&self, artist: &Artist) -> Vec<Album> {
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
        eprintln!("Found {} {} albums", albums.len(), artist.name);

        albums
    }
}
