#[macro_use]
extern crate clap;
extern crate nix;

use clap::App;
use nix::unistd::{chdir, getcwd};

fn main() {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(ref matches) = matches.subcommand_matches("run") {
        println!("cwd is {:?}", getcwd().unwrap());
        let rootfs = matches.value_of("rootfs").unwrap();
        chdir(rootfs).unwrap();
        println!("cwd is {:?}", getcwd().unwrap());
    }
}
