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
use nix::fcntl::{open, OFlag};
use nix::unistd::chdir;
use nix::unistd::{fork, ForkResult, execvp, close};
use nix::sched::CloneFlags;
use nix::sched::{setns, unshare};
use nix::sys::stat::Mode;
use nix::sys::wait::wait;
use nix_extension::clearenv;
use oci::Spec;
use std::collections::HashMap;
use std::ffi::CString;
use std::fs::File;
use std::fs::create_dir;
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
    run().expect("Failed exec minicon");
}

fn run() -> Result<()> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let state_dir = matches.value_of("root").unwrap().to_string();

    match matches.subcommand() {
        ("create", Some(create_matches)) => {
            cmd_create(
                create_matches.value_of("id").unwrap(), 
                &state_dir,
                create_matches
            )  
        }
        ("run", Some(run_matches)) => {
            cmd_run(run_matches) 
        }
        _ => bail!("Command nod recognized."), 
    }
}

fn container_dir(root: &str, id: &str) -> String {
    format!("{}/{}", root, id)  
}

fn cmd_create(id: &str, state_dir: &str, matches: &ArgMatches) -> Result<()> {
    initialize(&NAMESPACES);

    let bundle = matches.value_of("bundle").unwrap();
    chdir(&*bundle).chain_err(|| format!("Failed to chdir {}", bundle))?;

    let dir = container_dir(state_dir, id);
    create_dir(&dir).chain_err(|| format!("Failed create dir {}", dir))?;

    let spec = read_config("config.json")?;
    
    match fork()? {
        ForkResult::Child => {
            let mut clone_flag = CloneFlags::empty();
            let mut to_enter = Vec::new();
            for ns in spec.linux.namespaces {
                // NOTE: Do namespaces type duplicate to occure error

                if let Some(namespace) = NAMESPACES.get(&*ns.typ) {
                    if ns.path.is_empty() {
                        clone_flag.insert(*namespace);  
                    } else {
                        let fd = open(&*ns.path, OFlag::empty(), Mode::empty()).chain_err(|| format!("Failed to open file {}", ns.path))?;
                        to_enter.push((*namespace, fd));  
                    } 
                }
            }

            for &(namespace, fd) in &to_enter {
                setns(fd, namespace).chain_err(|| format!("Failed to enter {:?}", namespace))?;
                close(fd)?;  
            }
            unshare(clone_flag)?; 
            
            // NOTE: Do get arg
            // let a: [String; 1] = ["test".to_string()];
            // exec(matches.value_of("command").unwrap(), &a, &a)?;
        }
        ForkResult::Parent { .. } => {
            wait()?;
        }
    }
    Ok(())  
}

fn cmd_run(_matches: &ArgMatches) -> Result<()> {
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
