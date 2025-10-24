use std::{
    collections::HashMap,
    fs::{self},
    io::{Write, stdin, stdout},
    iter,
};

use clap::Parser;

use crate::artist::{Artist, Music};

mod artist;

#[derive(Parser, Debug)]
enum Args {
    Search { artist: String },
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

fn main() {
    let music_api = Music::new();

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
                Args::Search { artist } => {
                    let mut artist = music_api
                        .search(&artist)
                        .unwrap()
                        .artists
                        .into_iter()
                        .next()
                        .unwrap();
                    music_api.fetch_songs(&mut artist);
                    artists.insert(artist.name.clone(), artist);
                }
                Args::List => {
                    println!("{:#?}", artists);
                }
            },
            Err(e) => eprintln!("{}", e.render()),
        }

        fs::write("save.json", serde_json::to_string_pretty(&artists).unwrap()).unwrap();
    }
}
