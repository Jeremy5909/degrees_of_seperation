use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

pub type ArtistSmall = HashMap<String, String>;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub collaborators: Option<ArtistSmall>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Album {
    pub(crate) name: String,
    pub(crate) id: String,
    pub(crate) tracks: Option<Vec<Song>>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Song {
    pub(crate) name: String,
    pub(crate) id: String,
    pub(crate) artists: Vec<Artist>,
}

#[derive(Debug)]
pub(super) enum Entity {
    Albums,
    Artists,
    Songs,
}
impl Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let a = match self {
            Self::Albums => "albums",
            Self::Artists => "artists",
            Self::Songs => "tracks",
        };
        write!(f, "{a}")
    }
}
