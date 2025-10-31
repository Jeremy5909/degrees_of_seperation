use std::{
    collections::{HashMap, HashSet},
    fs::{self},
    io::{Write, stdin, stdout},
    iter,
};

use clap::Parser;
use fuzzy_match::fuzzy_match;

use crate::artist::{Artist, Music};

mod artist;

#[derive(Parser, Debug)]
enum Args {
    Search {
        artist: String,
        #[arg(default_value_t = 0, short = 'n')]
        recursion: usize,
    },
    Delete {
        artist: String,
    },
    List,
}

fn read_command() -> Vec<String> {
    print!("> ");
    stdout().flush().unwrap();
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    let input = input.trim();
    shlex::split(input).unwrap()
}

async fn search(
    name: String,
    n: usize,
    music_api: &Music,
    visited: &mut HashSet<String>,
) -> Vec<Artist> {
    if visited.contains(&name) {
        return Vec::new();
    }
    visited.insert(name.clone());

    let mut total_artists = Vec::new();
    let artist = match music_api.search_artist(&name).await {
        Ok(artist) => artist,
        Err(_) => return Vec::new(),
    };
    let collabs = artist.collaborators.clone().unwrap_or_default();
    total_artists.push(artist);
    if n == 0 {
        return total_artists;
    }

    for (collab_name, _) in collabs {
        let mut collabs = Box::pin(search(collab_name, n - 1, music_api, visited)).await;
        total_artists.append(&mut collabs);
    }

    total_artists
}

#[tokio::main]
async fn main() {
    let music_api = Music::new().await;

    let save_file = fs::read_to_string("save.json").unwrap_or_default();

    let mut artists: HashMap<String, Artist> = if save_file.is_empty() {
        HashMap::new()
    } else {
        serde_json::from_str(&save_file).unwrap()
    };

    loop {
        let args = read_command();
        let args: Vec<_> = args.iter().map(|s| s.as_str()).collect();

        match Args::try_parse_from(iter::once(">").chain(args)) {
            Ok(command) => match command {
                Args::Search { artist, recursion } => {
                    let mut visited = HashSet::new();
                    let all_artists = search(artist, recursion, &music_api, &mut visited).await;
                    artists.extend(
                        all_artists
                            .into_iter()
                            .map(|artist| (artist.name.clone(), artist)),
                    );
                }
                Args::Delete { artist } => {
                    if let Some(matched_artist) = fuzzy_match(
                        &artist,
                        artists.clone().iter().map(|(name, id)| (name.as_str(), id)),
                    ) {
                        println!("Delete '{}'? y/n", matched_artist.name);
                        let mut answer = String::new();
                        stdin().read_line(&mut answer).unwrap();
                        match answer.trim() {
                            "y" | "yes" => {
                                artists.remove(&matched_artist.name);
                                println!("'{}' deleted", matched_artist.name);
                            }
                            _ => println!("Canceled"),
                        }
                    } else {
                        println!("Artist not found");
                    }
                }
                Args::List => {
                    println!(
                        "{}",
                        artists
                            .iter()
                            .map(|artist| artist.0.clone())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
            },
            Err(e) => eprintln!("{}", e.render()),
        }

        fs::write("save.json", serde_json::to_string_pretty(&artists).unwrap()).unwrap();
    }
}
