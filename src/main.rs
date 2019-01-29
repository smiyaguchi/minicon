#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
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
use nix::sched::{setns, unshare};
use nix::sched::CloneFlags;
use nix::sys::socket::{bind, connect, socket};
use nix::sys::socket::{AddressFamily, SockAddr, SockType, SockFlag, UnixAddr};
use nix::sys::stat::Mode;
use nix::unistd::{chdir, close, dup2, fork, pipe2, write};
use nix::unistd::ForkResult;
use oci::{Linux, NamespaceType, Mapping, Spec};
use std::collections::HashMap;
use std::fs::{canonicalize, create_dir, create_dir_all, remove_dir_all};
use std::os::unix::fs::symlink;
use std::os::unix::io::RawFd;

const TSOCKETFD: RawFd = 9;

lazy_static! {
    static ref NAMESPACES: HashMap<NamespaceType, CloneFlags> = {
        let mut mapping = HashMap::new();
        mapping.insert(NamespaceType::ipc, CloneFlags::CLONE_NEWIPC);
        mapping.insert(NamespaceType::uts, CloneFlags::CLONE_NEWUTS);
        mapping.insert(NamespaceType::network, CloneFlags::CLONE_NEWNET);
        mapping.insert(NamespaceType::pid, CloneFlags::CLONE_NEWPID);
        mapping.insert(NamespaceType::mount, CloneFlags::CLONE_NEWNS);
        mapping.insert(NamespaceType::cgroup, CloneFlags::CLONE_NEWCGROUP);
        mapping.insert(NamespaceType::user, CloneFlags::CLONE_NEWUSER);
        mapping
    };    
}

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
    let mut userns = false;
    let linux = spec.linux.as_ref().unwrap();

    for ns in &linux.namespaces {
        if ns.path.is_empty() {
            if let Some(v) = NAMESPACES.get(&ns.typ) {
                clone_flags.insert(*v);    
            }
        } else {
            let fd = open(&*ns.path, OFlag::empty(), Mode::empty())?;

            if ns.typ == NamespaceType::pid {
                setns(fd, CloneFlags::CLONE_NEWPID)?;
                close(fd)?;
                continue;    
            }
            
            if let Some(v) = NAMESPACES.get(&ns.typ) {
                ns_enter.push((fd, *v));
            }
        }
    }
    
    let mut pidns = false;
    if clone_flags.contains(CloneFlags::CLONE_NEWPID) {
        pidns = true; 
    }

    let (_chid_pid, _wfd) = do_fork(userns, &spec, &linux)?;

    for (fd, clone_flag) in ns_enter {
        setns(fd, clone_flag)?;
        close(fd)?;    
    }

    unshare(clone_flags)?;

    if pidns {
        fork_pidns()?;
    }

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

fn do_fork(userns: bool, _spec: &Spec, linux: &Linux) -> Result<(i32, RawFd), Error> {
    let (rfd, wfd) = pipe2(OFlag::O_CLOEXEC)?;
    match fork()? {
        ForkResult::Child => {
            close(rfd)?;
            
            if userns {
                unshare(CloneFlags::CLONE_NEWUSER)?;    
            }    
        },
        ForkResult::Parent { child } => {
            close(wfd)?;
          
            if userns {
                write_id_mappings(&format!("/proc/{}/uid_map", child), &linux.uid_mappings)?;
                write_id_mappings(&format!("/proc/{}/gid_map", child), &linux.gid_mappings)?;
            }
            
            std::process::exit(0);   
        }    
    }
    Ok((-1, wfd))    
}

fn write_id_mappings(path: &str, id_mappings: &Vec<Mapping>) -> Result<(), Error> {
    let mut data = String::new();
    for m in id_mappings {
        let value = format!("{} {} {}\n", m.container_id, m.host_id, m.size);
        data = data + &value;
    }

    if !data.is_empty() {
        let fd = open(path, OFlag::O_WRONLY, Mode::empty())?;
        write(fd, data.as_bytes())?;
        close(fd)?;    
    }
    Ok(())
}

fn fork_pidns() -> Result<(), Error> {
    match fork()? {
        ForkResult::Child => {
            // continue process
        },
        ForkResult::Parent { .. } => {
            std::process::exit(0);    
        } 
    }
    Ok(())    
}
