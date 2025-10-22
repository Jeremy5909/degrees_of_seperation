use std::{
    fs::File,
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

    let mut save_file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open("save.json")
        .unwrap();
    save_file.seek(SeekFrom::Start(0)).unwrap();

    let mut artists: Vec<_> = serde_json::from_reader(&mut save_file).unwrap_or(Vec::new());

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

        save_file.set_len(0).unwrap();
        save_file.seek(SeekFrom::Start(0)).unwrap();
        serde_json::to_writer(&mut save_file, &artists).unwrap();
    }
}
