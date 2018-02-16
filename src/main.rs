#[macro_use]
extern crate clap;
extern crate nix;

use clap::{App, ArgMatches};
use nix::unistd::{chdir, getcwd};

fn main() {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(ref matches) = matches.subcommand_matches("run") {
        match command_run(&matches) {
            Ok(()) => println!("Success chdir"),
            Err(e) => println!("{}", e),  
        }
        println!("cwd is {:?}", getcwd().unwrap());
    }
}

fn command_run(matches: &ArgMatches) -> Result<(), String> {
    let rootfs = matches.value_of("rootfs").unwrap();
    chdir(rootfs).expect("Failed to chdir");
    Ok(())  
}
