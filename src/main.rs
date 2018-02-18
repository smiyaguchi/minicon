#![recursion_limit = "1024"]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate libc;
extern crate nix;

mod errors;
mod nix_extension;

use clap::{App, ArgMatches};
use errors::*;
use nix::unistd::chdir;
use nix::unistd::{fork, ForkResult, execvp};
use nix::sched::CloneFlags;
use nix::sched::unshare;
use nix::sys::wait::wait;
use nix_extension::clearenv;
use std::ffi::CString;

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
    let root = matches.value_of("root").unwrap();
    chdir(root).chain_err(|| format!("Failed to chdir {}", root))?;
    match fork()? {
        ForkResult::Child => {
            unshare(CloneFlags::CLONE_NEWNS | 
                    CloneFlags::CLONE_NEWIPC | 
                    CloneFlags::CLONE_NEWNET | 
                    CloneFlags::CLONE_NEWPID | 
                    CloneFlags::CLONE_NEWUTS | 
                    CloneFlags::CLONE_NEWUSER | 
                    CloneFlags::CLONE_NEWCGROUP)
                .chain_err(|| "Failed unshare")?;
            
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