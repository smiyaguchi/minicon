use lazy_static::initialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn init() {
    initialize(&MOUNTS);
}

lazy_static! {
    pub static ref MOUNTS: HashMap<String, String> = {
        let mut mounts = HashMap::new();
        let f = match File::open("/proc/self/mountinfo") {
            Ok(f) => f,
            Err(e) => {
                return mounts;
            }
        };
        for line in BufReader::new(f).lines() {
            let l = match line {
                Ok(l) => l,
                Err(e) => {
                    return mounts;
                }
            };
        }
        mounts
    };
}
