use std::fs::File;
use std::io::Read;

use anyhow::Result;
use clap::{Arg, ArgAction, Command};

fn main() {
    let matches = Command::new("Emulator")
        .arg(Arg::new("file").required(true))
        .arg(Arg::new("debug").short('d').action(ArgAction::SetTrue))
        .get_matches();
    let file_name: &String = matches.get_one("file").unwrap();
    let debug = matches.get_flag("debug");

    println!("Loading {}", file_name);

    let mut buf = vec![];
    File::open(file_name)
        .and_then(|mut file| file.read_to_end(&mut buf))
        .unwrap();
    run(&buf, debug).unwrap()
}

fn run(_data: &[u8], _debug: bool) -> Result<()> {
    Ok(())
}
