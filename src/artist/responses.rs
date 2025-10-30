use std::vec;

use serde::{Deserialize, Serialize};

use crate::artist::{Album, Artist, Track};

#[derive(Serialize, Deserialize)]
pub(super) struct ArtistsResponse {
    pub artists: Artists,
}
impl IntoIterator for ArtistsResponse {
    type Item = Artist;

    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.artists.items.into_iter()
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct Artists {
    pub items: Vec<Artist>,
}

#[derive(Deserialize)]
pub(super) struct Albums {
    items: Vec<Album>,
}
impl IntoIterator for Albums {
    type Item = Album;

    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub(super) struct Tracks {
    items: Vec<Track>,
}
impl IntoIterator for Tracks {
    type Item = Track;

    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}
