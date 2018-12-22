#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
extern crate nix;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod oci;
mod error;

use clap::{App, ArgMatches};
use failure::Error;
use nix::fcntl::{open, OFlag};
use nix::sched::{setns, unshare, CloneFlags};
use nix::sys::socket::{bind, connect, socket, AddressFamily, SockAddr, SockType, SockFlag, UnixAddr};
use nix::sys::stat::Mode;
use nix::unistd::{chdir, close, dup2};
use oci::{Spec, NamespaceType};
use std::fs::{canonicalize, create_dir, create_dir_all, remove_dir_all};
use std::os::unix::fs::symlink;
use std::os::unix::io::RawFd;

const TSOCKETFD: RawFd = 9;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    match run(&matches) {
        Ok(_) => println!("success"),
        Err(e) => println!("{}", e),    
    }
}

fn run(matches: &ArgMatches) -> Result<(), Error> {
    let state_dir = "/run/minicon";
    create_dir_all(state_dir)?;

    match matches.subcommand() {
        ("create", Some(matches)) => create(
            state_dir,
            matches.value_of("id").unwrap(),
            matches,
        ),
        _ => panic!("error"),
    }    
}

fn create(state_dir: &str, id: &str, matches: &ArgMatches) -> Result<(), Error> {
    let bundle = matches.value_of("bundle").unwrap();
    chdir(bundle)?;

    let dir = instance_dir(state_dir, id);
    if let Err(e) = create_dir(&dir) {
        panic!("container id {} is already exsits. {}", id, e);            
    }
    
    if let Err(e) = create_container(&*dir, id, matches) {
        let _ = remove_dir_all(&dir);
        Err(e)
    } else {
        Ok(())    
    }
}

fn instance_dir(state_dir: &str, id: &str) -> String {
    format!("{}/{}", state_dir, id)    
}

fn create_container(instance_dir: &str, _id: &str, matches: &ArgMatches) -> Result<(), Error> {
    let spec = Spec::load()?;
    let _rootfs = canonicalize(&spec.root.path)?
        .to_string_lossy()
        .into_owned();

    chdir(&*instance_dir)?;     

    let csocket = matches.value_of("console-socket").unwrap();
    if csocket != "" {
        let lnk = format!("{}/console-socket", instance_dir);
        symlink(&csocket, lnk)?;    
    }

    let console = matches.value_of("console").unwrap();
    if  console != "" {
        let lnk = format!("{}/console", instance_dir);
        symlink(&console, lnk)?;    
    };

    let (_csocketfd, _consolefd, _tsocketfd) = if !matches.is_present("no-trigger") {
        let tsocket = "trigger-socket";
        let tmpfd = socket(AddressFamily::Unix, SockType::Stream, SockFlag::empty(), None)?;   
        
        dup2(tmpfd, TSOCKETFD)?;
        close(tmpfd)?;
        let tsocketfd = TSOCKETFD;
        bind(tsocketfd, &SockAddr::Unix(UnixAddr::new(&*tsocket)?))?;
        let (csocketfd, consolefd) = load_console_socket()?;
        (csocketfd, consolefd, tsocketfd)
    } else {
        (-1, -1, -1)    
    };
    
    let mut clone_flags = CloneFlags::empty();
    let mut ns_enter = Vec::new();
    let mut is_pid = false;
    let linux = &spec.linux.unwrap();

    for ns in &linux.namespaces {
        if ns.path.is_empty() {
            match ns.typ {
                NamespaceType::pid => clone_flags.insert(CloneFlags::CLONE_NEWPID),
                NamespaceType::network => clone_flags.insert(CloneFlags::CLONE_NEWNET),
                NamespaceType::mount => clone_flags.insert(CloneFlags::CLONE_NEWNS),
                NamespaceType::ipc => clone_flags.insert(CloneFlags::CLONE_NEWPID),
                NamespaceType::uts => clone_flags.insert(CloneFlags::CLONE_NEWUTS),
                NamespaceType::user => clone_flags.insert(CloneFlags::CLONE_NEWUSER),
                NamespaceType::cgroup => clone_flags.insert(CloneFlags::CLONE_NEWCGROUP),
            }
        } else {
            let fd = open(&*ns.path, OFlag::empty(), Mode::empty())?;
            
            match ns.typ {
                NamespaceType::pid => {
                    setns(fd, CloneFlags::CLONE_NEWPID)?;
                    close(fd)?;
                    continue;    
                },
                _ => ns_enter.push(fd),    
            }
        }

        if clone_flags.contains(CloneFlags::CLONE_NEWPID) {
            is_pid = true; 
        }
    }

    unshare(clone_flags)?;

    Ok(())    
}

fn load_console_socket() -> Result<(RawFd, RawFd), Error> {
    let csocket = "console-socket";
    let mut csocketfd = socket(AddressFamily::Unix, SockType::Stream, SockFlag::empty(), None)?;
    csocketfd = match connect(csocketfd, &SockAddr::Unix(UnixAddr::new(&*csocket)?)) {
            Ok(()) => csocketfd,
            Err(_) => -1,
        };
        
    let console = "console";
    let consolefd = match open(&*console, OFlag::O_NOCTTY | OFlag::O_RDWR, Mode::empty()) {
            Ok(fd) => fd,
            Err(_) => -1,  
        };

    Ok((csocketfd, consolefd))
}

