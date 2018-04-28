#![allow(non_camel_case_types)]
#![recursion_limit = "1024"]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate nix;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod cgroup;
mod container;
mod errors;
mod mount;
mod nix_extension;
mod oci;
mod pipe;

use clap::{App, ArgMatches};
use container::{Container, Operation};
use errors::*;

fn main() {
    run().expect("Failed run minicon");
}

fn run() -> Result<()> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let state_dir = matches.value_of("root").unwrap().to_string();

    let mut container = Container::new();

    match matches.subcommand() {
        ("state", Some(state_matches)) => {
            container.state(
                state_matches.value_of("id").unwrap(),
                &state_dir
            )
        }
        ("create", Some(create_matches)) => {
            container.create(
                create_matches.value_of("id").unwrap(),
                create_matches.value_of("bundle").unwrap(),
                &state_dir
            )
        }
        ("start", Some(start_matches)) => {
            container.start(
                start_matches.value_of("id").unwrap(),
                &state_dir
            )
        }
        ("kill", Some(kill_matches)) => {
            container.kill(
                kill_matches.value_of("id").unwrap(),
                &state_dir,
                kill_matches.value_of("signal").unwrap()
            )
        }
        ("delete", Some(delete_matches)) => {
            container.delete(
                delete_matches.value_of("id").unwrap(),
                &state_dir
            )
        }
        _ => bail!("Command not recognized."), 
    }
}
