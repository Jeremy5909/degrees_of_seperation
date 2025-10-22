use std::{
    fs::{self, File},
    io::{Seek, SeekFrom, Write, stdin, stdout},
    iter,
};

use clap::Parser;
use reqwest::blocking::Client;

use crate::artist::search_artist;

mod artist;

#[derive(Parser, Debug)]
enum Args {
    Load { artist: String },
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
    let mut artists: Vec<_> = serde_json::from_str(&save_file).unwrap_or(Vec::new());

    loop {
        let args = read_command();
        let args: Vec<_> = args.iter().map(|s| s.as_str()).collect();

        match Args::try_parse_from(iter::once(">").chain(args)) {
            Ok(command) => match command {
                Args::Load { artist } => {
                    let artist = search_artist(&web_client, &artist).unwrap();
                    println!("{artist:#?}");
                    artists.push(artist);
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
