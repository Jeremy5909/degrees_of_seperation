use std::{
    fs::File,
    io::{Write, stdin, stdout},
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
    Save,
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

    let mut save_file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open("save.json")
        .unwrap();

    let mut local_artists = Vec::new();

    loop {
        let args = read_command();
        let args: Vec<_> = args.iter().map(|s| s.as_str()).collect();

        match Args::try_parse_from(iter::once(">").chain(args)) {
            Ok(command) => match command {
                Args::Load { artist } => {
                    let artist = search_artist(&web_client, &artist).unwrap();
                    println!("{artist:#?}");
                    local_artists.push(artist);
                }
                Args::Save => match serde_json::to_writer(&mut save_file, &local_artists) {
                    Ok(()) => println!("Saved "),
                    Err(e) => println!("{}", e),
                },
                Args::List => {
                    println!("{:#?}", local_artists);
                }
            },
            Err(e) => eprintln!("{}", e.render()),
        }
    }
}
