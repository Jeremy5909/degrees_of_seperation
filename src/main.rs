use std::{
    collections::HashMap,
    fs::{self},
    io::{Write, stdin, stdout},
    iter,
};

use clap::Parser;
use fuzzy_match::fuzzy_match;

use crate::artist::{Music, entities::Artist};

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
                    let visited = Default::default();
                    let all_artists = music_api.search_recursive(artist, recursion, visited).await;
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
        eprintln!("Saved!");
    }
}
