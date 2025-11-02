use std::vec;

use serde::Deserialize;

use crate::music::entities::{Album, Artist, Song};

/////////////
// Artists //
/////////////
#[derive(Deserialize)]
pub(super) struct Artists {
    artists: InnerArtists,
}
impl IntoIterator for Artists {
    type Item = Artist;

    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.artists.into_iter()
    }
}
#[derive(Deserialize)]
struct InnerArtists {
    items: Vec<Artist>,
}
impl IntoIterator for InnerArtists {
    type Item = Artist;

    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

////////////
// Albums //
////////////
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

///////////
// Songs //
///////////
#[derive(Deserialize, Debug, Clone)]
pub(super) struct Songs {
    items: Vec<Song>,
}
impl IntoIterator for Songs {
    type Item = Song;

    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}
