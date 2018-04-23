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
use container::Container;
use container::Operation;
use errors::*;
use nix::unistd::chdir;
use nix::unistd::{write, close};
use nix::sys::socket::{socket, bind, listen, accept, connect};
use nix::sys::socket::{SockAddr, UnixAddr, AddressFamily, SockType, SockFlag};
use nix::sys::stat::Mode;
use oci::{Spec, IDMapping};
use pipe::Pipe;
use std::collections::HashMap;
use std::ffi::CString;
use std::fs::File;
use std::fs::{create_dir_all, canonicalize};
use std::io::Write;
use std::os::unix::io::RawFd;
use std::path::Path;

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
            cmd_state(
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
            cmd_start(
                start_matches.value_of("id").unwrap(),
                &state_dir
            )  
        }
        ("kill", Some(kill_matches)) => {
            cmd_kill(
                kill_matches.value_of("id").unwrap(),
                kill_matches.value_of("signal").unwrap(),
                &state_dir
            )
        }
        ("delete", Some(delete_matches)) => {
            cmd_delete(
                delete_matches.value_of("id").unwrap(),
                &state_dir
            )
        }
        _ => bail!("Command not recognized."), 
    }
}

fn container_dir(root: &str, id: &str) -> String {
    format!("{}/{}", root, id)  
}

fn cmd_state(id: &str, state_dir: &str) -> Result<()> {
    let dir = container_dir(state_dir, id);
    chdir(&*dir).chain_err(|| format!("Failed to chdir {}", dir))?;
    
    Ok(())       
}

fn cmd_start(id: &str, state_dir: &str) -> Result<()> {
    let dir = container_dir(state_dir, id);
    chdir(&*dir).chain_err(|| format!("Failed change dir {}", dir))?;

    let socket_url = "endpoint";
    let sfd = socket(AddressFamily::Unix, SockType::Stream, SockFlag::empty(), None)?;
    connect(sfd, &SockAddr::Unix(UnixAddr::new(&*socket_url)?))?;
    let data: &[u8] = &[0];
    write(sfd, data).chain_err(|| "Failed to write socket")?;
    close(sfd)?;

    Ok(())
}

fn cmd_kill(id: &str, signal: &str, state_dir: &str) -> Result<()> {
    Ok(())
}

fn cmd_delete(id: &str, state_dir: &str) -> Result<()> {
    Ok(())
}
