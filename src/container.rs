use cgroup;
use clap::{App, ArgMatches};
use errors::*;
use lazy_static::initialize;
use mount::do_pivot_root;
use nix::fcntl::{open, OFlag};
use nix::unistd::{chdir, sethostname};
use nix::unistd::{fork, ForkResult, execvp, read, write, close, pipe2, setsid, Pid};
use nix::sched::CloneFlags;
use nix::sched::{setns, unshare};
use nix::sys::socket::{socket, bind, listen, accept, connect};
use nix::sys::socket::{SockAddr, UnixAddr, AddressFamily, SockType, SockFlag};
use nix::sys::stat::Mode;
use nix_extension::{clearenv, putenv, setrlimit};
use oci::{Spec, IDMapping};
use pipe;
use serde_json;
use std;
use std::collections::HashMap;
use std::ffi::CString;
use std::fs::File;
use std::fs::{create_dir_all, canonicalize};
use std::io::Write;
use std::os::unix::io::RawFd;
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

#[derive(Debug)]
pub struct Container {
    pub pid: Pid    
}

pub trait Operation {
    fn create(&mut self, container_id: &str, bundle: &str, root: &str) -> Result<()>;
    fn start(&mut self, container_id: &str, root: &str) -> Result<()>;
    fn state(&mut self, container_id: &str, root: &str) -> Result<()>;
    fn kill(&mut self, container_id: &str, root: &str, signal: &str) -> Result<()>;
    fn delete(&mut self, container_id: &str, root: &str) -> Result<()>;
}

impl Container {
    pub fn new() -> Container {
        Container {
            pid: Pid::from_raw(0),
        }
    }
}

impl Operation for Container {
    fn create(&mut self, container_id: &str, bundle: &str, root: &str) -> Result<()> {
        initialize(&NAMESPACES);

        chdir(&*bundle).chain_err(|| format!("Failed to chdir {}", bundle))?;

        let dir = container_dir(root, container_id);
        create_dir_all(&dir).chain_err(|| format!("Failed create dir {}", dir))?;
    
        run_container(&dir)?;
        Ok(())
    }

    fn start(&mut self, container_id: &str, root: &str) -> Result<()> {
        let dir = container_dir(root, container_id);
        chdir(&*dir).chain_err(|| format!("Failed change dir {}", dir))?;

        let socket_url = "endpoint";
        let sfd = socket(AddressFamily::Unix, SockType::Stream, SockFlag::empty(), None)?;
        connect(sfd, &SockAddr::Unix(UnixAddr::new(&*socket_url)?))?;
        let data: &[u8] = &[0];
        write(sfd, data).chain_err(|| "Failed to write socket")?;
        close(sfd)?;

        Ok(())
    }

    fn state(&mut self, container_id: &str, root: &str) -> Result<()> {
        let dir = container_dir(root, container_id);
        chdir(&*dir).chain_err(|| format!("Failed change dir {}", dir))?;

        Ok(())
    }

    fn kill(&mut self, container_id: &str, root: &str, signal: &str) -> Result<()> {
        Ok(())
    }

    fn delete(&mut self, container_id: &str, root: &str) -> Result<()> {
        Ok(())  
    }
}

fn container_dir(root: &str, id: &str) -> String {
    format!("{}/{}", root, id)  
}

fn run_container(container_dir: &str) -> Result<()> {
    let spec = read_config("config.json")?;

    chdir(&*container_dir).chain_err(|| format!("Failed to chdir {}", container_dir))?;

    let mut clone_flag = CloneFlags::empty();
    let mut to_enter = Vec::new();
    let mut userns = false;

    for ns in &spec.linux.namespaces {
        if let Some(namespace) = NAMESPACES.get(&*ns.typ) {
            if ns.path.is_empty() {
                if ns.typ == "user" {
                    userns = true;
                    continue;  
                }
                clone_flag.insert(*namespace);  
            } else {
                let fd = open(&*ns.path, OFlag::empty(), Mode::empty()).chain_err(|| format!("Failed to open file {}", ns.path))?;
                
                if ns.typ == "pid" {
                    setns(fd, CloneFlags::CLONE_NEWPID)?;   
                    close(fd)?; 
                    continue;
                } 
                to_enter.push((*namespace, fd)); 
            }
        }
    }

    let mut pidns = false;
    if clone_flag.contains(CloneFlags::CLONE_NEWPID) {
        pidns = true;
    }

    let (child_pid, _wfd) = do_fork(userns, &spec)?;

    if child_pid != -1 {
        return Ok(())  
    }

    for &(namespace, fd) in &to_enter {
        setns(fd, namespace).chain_err(|| "Failed to setns")?;
        close(fd)?;  
    }
    unshare(clone_flag).chain_err(|| "Failed to unshare namespace")?;

    if pidns {
        do_fork_pidns()?;  
    }

    if clone_flag.contains(CloneFlags::CLONE_NEWUTS) {
        sethostname(&spec.hostname)?;
    }

    if clone_flag.contains(CloneFlags::CLONE_NEWNS) {
        let rootfs = canonicalize(&spec.root.path).chain_err(|| "Failed to canonicalize")?
                        .to_string_lossy()
                        .into_owned();
        do_pivot_root(&*rootfs)?;
    }

    setsid()?;

    let socket_url = "endpoint";
    let sfd = socket(AddressFamily::Unix, SockType::Stream, SockFlag::empty(), None)?;
    bind(sfd, &SockAddr::Unix(UnixAddr::new(&*socket_url)?))?;
    listen(sfd, 1)?;
    let afd = accept(sfd)?;
    let data: &mut [u8] = &mut [0];
    read(afd, data)?;

    close(afd)?;
    close(sfd)?;

    chdir(&*spec.process.cwd)?;
    
    do_exec(&spec.process.args[0], &spec.process.args, &spec.process.env)?; 
        
    Ok(())  
}

fn do_fork(userns: bool, spec: &Spec) -> Result<(i32, RawFd)> {
    let child_pipe = pipe::Pipe::new().chain_err(|| "Failed to create child pipe")?;
    let parent_pipe = pipe::Pipe::new().chain_err(|| "Failed to create parent pipe")?;
    let (rfd, wfd) = pipe2(OFlag::O_CLOEXEC).chain_err(|| "Failed to create pipe")?;
    match fork()? {
        ForkResult::Child => {
            close(rfd).chain_err(|| "Failed to close rfd")?;

            let p = &spec.process;
            if let Some(adj) = p.oom_score_adj {
                let mut f = File::create("/proc/self/oom_score_adj")?;
                f.write_all(adj.to_string().as_bytes())?;
            }

            for rlimit in &spec.process.rlimits {
                setrlimit(rlimit.typ as i32, rlimit.soft, rlimit.hard)?;
            }

            if userns {
                unshare(CloneFlags::CLONE_NEWUSER).chain_err(|| "Failed to unshare user namespace")?;  
            }
            child_pipe.notify().chain_err(|| "Failed to notify parent")?;
            parent_pipe.wait().chain_err(|| "Failed to wait parent")?;
        }
        ForkResult::Parent { child } => {
            close(wfd).chain_err(|| "Faild to close wfd")?;
            child_pipe.wait().chain_err(|| "Failed to wait child")?; 

            if userns {
                write_id_mappings(&format!("/proc/{}/uid_map", child), &spec.linux.uid_mappings)?;
                write_id_mappings(&format!("/proc/{}/gid_map", child), &spec.linux.gid_mappings)?;
            }

            cgroup::init();

            parent_pipe.notify().chain_err(|| "Failed to notify child")?;

            std::process::exit(0);
        }  
    }
    Ok((-1, wfd))  
}

fn write_id_mappings(path: &str, id_mappings: &[IDMapping]) -> Result<()> {
    let mut data = String::new();
    for m in id_mappings {
        let value = format!("{} {} {}\n", m.container_id, m.host_id, m.size);
        data = data + &value;  
    }
    if !data.is_empty() {
        let fd = open(path, OFlag::O_WRONLY, Mode::empty())?;
        write(fd, data.as_bytes())?;  
        close(fd).unwrap();
    }
    Ok(()) 
}

fn do_fork_pidns() -> Result<()> {
    match fork()? {
        ForkResult::Child => {
            // continue process
        }
        ForkResult::Parent { .. } => {
            std::process::exit(0);
        }  
    } 
    Ok(()) 
}

fn read_config<P: AsRef<Path>>(path: P) -> Result<Spec> {
    let file = File::open(path)?;
    let spec = serde_json::from_reader(file)?;
    Ok(spec)      
}

fn do_exec(path: &str, args: &[String], env: &[String]) -> Result<()> {
    let p: CString = CString::new(path.to_string()).unwrap();
    let a: Vec<CString> = args.iter()
        .map(|s| CString::new(s.to_string()).unwrap_or_default())
        .collect();
    let env: Vec<CString> = env.iter()
        .map(|s| CString::new(s.to_string()).unwrap_or_default())
        .collect();
    clearenv()?;
    for e in &env {
        putenv(e)?;  
    }
    execvp(&p, &a).chain_err(|| "Failed to exec")?;
    Ok(())
}
