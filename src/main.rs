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

mod errors;
mod nix_extension;
mod oci;

use clap::{App, ArgMatches};
use errors::*;
use lazy_static::initialize;
use nix::unistd::chdir;
use nix::unistd::{fork, ForkResult, execvp};
use nix::sched::CloneFlags;
use nix::sched::unshare;
use nix::sys::wait::wait;
use nix_extension::clearenv;
use oci::Spec;
use std::collections::HashMap;
use std::ffi::CString;
use std::fs::File;
use std::path::Path;

lazy_static! {
    static ref NAMESPACES: HashMap<&'static str, CloneFlags> = {
        let mut n = HashMap::new();
        n.insert("ipc", CloneFlags::CLONE_NEWIPC);
        n.insert("uts", CloneFlags::CLONE_NEWUTS);
        n.insert("net", CloneFlags::CLONE_NEWNET);
        n.insert("pid", CloneFlags::CLONE_NEWPID);
        n.insert("mount", CloneFlags::CLONE_NEWNS);
        n.insert("cgroup", CloneFlags::CLONE_NEWCGROUP);
        n.insert("user", CloneFlags::CLONE_NEWUSER);
        n
    };  
}

fn main() {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(ref matches) = matches.subcommand_matches("run") {
        match command_run(&matches) {
          Ok(()) => println!("Success command run"),
          Err(e) => println!("{}", e),  
        }
    }
}

fn command_run(matches: &ArgMatches) -> Result<()> {
    let _spec = read_config("config.json")?;

    initialize(&NAMESPACES);

    let root = matches.value_of("root").unwrap();
    chdir(root).chain_err(|| format!("Failed to chdir {}", root))?;
    match fork()? {
        ForkResult::Child => {
            let mut clone_flag = CloneFlags::empty();
            //for n in spec.linux.namespaces {
            //    if let Some(flag) = NAMESPACES.get(n.typ) {
            //        clone_flag.insert(*flag);  
            //    }
            //}
            unshare(clone_flag).chain_err(|| "Failed unshare")?; 
            
            // NOTE: Do get arg
            let a: [String; 1] = ["test".to_string()];
            exec(matches.value_of("command").unwrap(), &a, &a)?;
        }
        ForkResult::Parent { .. } => {
            wait()?;
        }
    }
    Ok(())  
}

fn read_config<P: AsRef<Path>>(path: P) -> Result<Spec> {
    let file = File::open(path)?;
    let spec = serde_json::from_reader(file)?;
    Ok(spec)      
}

fn exec(path: &str, args: &[String], env: &[String]) -> Result<()> {
    clearenv()?;

    // NOTE: Do putenv

    let p: CString = CString::new(path.to_string()).unwrap();
    let a: Vec<CString> = args.iter()
        .map(|s| CString::new(s.to_string()).unwrap_or_default())
        .collect();
    let _env: Vec<CString> = env.iter()
        .map(|s| CString::new(s.to_string()).unwrap_or_default())
        .collect();
    execvp(&p, &a).chain_err(|| "Failed to exec")?;
    Ok(())
}
