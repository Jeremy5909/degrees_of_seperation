use std::{
    fs::{self},
    io::{Write, stdin, stdout},
    iter,
};

use clap::Parser;
use reqwest::blocking::Client;

use crate::artist::{Artists, search_artist};

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
    let web_client = Client::new();

    let save_file = fs::read_to_string("save.json").unwrap();
    let mut artists = serde_json::from_str(&save_file)
        .unwrap_or(Artists {
            artists: Vec::new(),
        })
        .into_hashmap();

    loop {
        let args = read_command();
        let args: Vec<_> = args.iter().map(|s| s.as_str()).collect();

        match Args::try_parse_from(iter::once(">").chain(args)) {
            Ok(command) => match command {
                Args::Search { artist } => {
                    let artist = search_artist(&web_client, &artist).unwrap();
                    println!("{artist:#?}");
                    artists.insert(artist.id.clone(), artist);
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
