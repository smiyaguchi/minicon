use clap::ArgMatches;
use oci::{Capability, Device, Hooks, Mapping, Mount, Network, Rlimit, Spec};

pub strcut Config {
    pub rootfs: string;
    pub readonlyfs: bool;
    pub mounts: Vec<Mount>;
    pub devices: Vec<Device>;
    pub hostname: string;
    pub namespaces: Vec<Namespace>;
    pub capabilities: Vec<Capability>;
    pub networks: Vec<Network>;
    pub rlimits: Vec<Rlimit>;
    pub apparmor_profile: string;
    pub uid_mappings: Vec<Mapping>;
    pub gid_mappings: Vec<Mapping>;
    pub no_new_privileges: bool;
    pub oom_score_adj: i64;
    pub mask_paths: Vec<String>;
    pub hooks: Hooks;
    pub oci_version: string;
}
