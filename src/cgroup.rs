use lazy_static::initialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn init() {
    initialize(&PATHS);
    initialize(&MOUNTS);
}

lazy_static! {
    pub static ref PATHS: HashMap<String, String> = {
        let mut paths = HashMap::new();
        let f = match File::open("/proc/self/cgroup") {
            Ok(f) => f,
            Err(_) => {
                return  paths;
            }
        };
        for line in BufReader::new(f).lines() {
            let l = match line {
                Ok(l) => l,
                Err(_) => {
                    return paths;
                }
            };
            let fields: Vec<&str> = l.split(':').collect();
            if fields.len() != 3 {
                continue;
            }
            paths.insert(fields[1].to_string(), fields[2].to_string());
        }
        paths
    };
}

lazy_static! {
    pub static ref MOUNTS: HashMap<String, String> = {
        let mut mounts = HashMap::new();
        let f = match File::open("/proc/self/mountinfo") {
            Ok(f) => f,
            Err(_) => {
                return mounts;
            }
        };
        for line in BufReader::new(f).lines() {
            let l = match line {
                Ok(l) => l,
                Err(_) => {
                    return mounts;
                }
            };
            if let Some(sep) = l.find(" - ") {
                if l.len() < sep + 10 {
                    continue;
                }
                let key = &l[sep+3..sep+10];
                if key != "cgroup " && key != "cgroup2" {
                    continue;
                }
                let pre: Vec<&str> = l[..sep].split(' ').collect();
                if pre.len() != 7 {
                    continue;
                }
                let post: Vec<&str> = l[sep+3..].split(' ').collect();
                if post.len() != 3 {
                    continue;
                }
                let mut offset = post[2].len();
                while let Some(o) = post[2][..offset].rfind(',') {
                    let name = &post[2][o+1..];
                    if PATHS.contains_key(name) {
                        mounts.insert(name.to_string(), pre[4].to_string());
                        break;
                    }
                    offset = o;
                }
            } else {
                // Do nothing
            }
        }
        mounts
    };
}
