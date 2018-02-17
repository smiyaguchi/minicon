#![recursion_limit = "1024"]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate nix;

mod errors;

use clap::{App, ArgMatches};
use errors::*;
use nix::unistd::chdir;

fn main() {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(ref matches) = matches.subcommand_matches("run") {
        match command_run(&matches) {
            Ok(()) => println!("Success chdir"),
            Err(e) => println!("{}", e),  
        }
    }
}

fn command_run(matches: &ArgMatches) -> Result<()> {
    let root = matches.value_of("root").unwrap();
    chdir(root).chain_err(|| format!("Failed to chdir {}", root))?;
    Ok(())  
}
