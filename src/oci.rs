#![allow(non_camel_case_types)]
use std::collections::HashMap;
use failure::Error;
use serde_json::from_reader;
use std::fs::File;
use std::io::BufReader;

#[derive(Serialize, Deserialize, Debug)]
pub struct Spec {
    #[serde(default, rename = "ociVersion")]
    pub oci_version: String,
    pub root: Root,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mounts: Vec<Mount>,
    pub process: Process,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub hostname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linux: Option<Linux>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<Hooks>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub annotations: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub readonly: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mount {
    #[serde(default)]
    pub destination: String,
    #[serde(default, skip_serializing_if = "String::is_empty", rename = "type")]
    pub typ: String,
    #[serde(default)]
    pub source: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Process {
    #[serde(default)]
    pub terminal: bool,
    #[serde(default, rename = "consoleSize")]
    pub console_size: ConsoleSize,
    #[serde(default)]
    pub cwd: String,
    #[serde(default)]
    pub env: Vec<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub rlimits: Vec<Rlimit>,
    #[serde(default, rename = "apparmorProfile")]
    pub apparmor_profile: String,
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    #[serde(default, rename = "noNewPrivileges")]
    pub no_new_privileges: bool,
    #[serde(default, rename = "oomScoreAdj")]
    pub oom_score_adj: i64,
    #[serde(default, rename = "selinuxLabel")]
    pub selinux_label: String,
    pub user: User,
}

#[derive(Default, PartialEq, Serialize, Deserialize, Debug)]
pub struct ConsoleSize {
    #[serde(default)]
    pub height: u64,
    #[serde(default)]
    pub width: u64,    
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rlimit {
    #[serde(default, rename = "type")]
    pub typ: String,
    #[serde(default)]
    pub soft: u64,
    #[serde(default)]
    pub hard: u64,    
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Capability {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effective: Vec<CapabilityType>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bounding: Vec<CapabilityType>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inheritable: Vec<CapabilityType>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permitted: Vec<CapabilityType>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ambient: Vec<CapabilityType>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[repr(u8)]
pub enum CapabilityType {
    CAP_CHOWN,
    CAP_DAC_OVERRIDE,
    CAP_DAC_READ_SEARCH,
    CAP_FOWNER,
    CAP_FSETID,
    CAP_KILL,
    CAP_SETGID,
    CAP_SETUID,
    CAP_SETPCAP,
    CAP_LINUX_IMMUTABLE,
    CAP_NET_BIND_SERVICE,
    CAP_NET_BROADCAST,
    CAP_NET_ADMIN,
    CAP_NET_RAW,
    CAP_IPC_LOCK,
    CAP_IPC_OWNER,
    CAP_SYS_MODULE,
    CAP_SYS_RAWIO,
    CAP_SYS_CHROOT,
    CAP_SYS_PTRACE,
    CAP_SYS_PACCT,
    CAP_SYS_ADMIN,
    CAP_SYS_BOOT,
    CAP_SYS_NICE,
    CAP_SYS_RESOURCE,
    CAP_SYS_TIME,
    CAP_SYS_TTY_CONFIG,
    CAP_MKNOD,
    CAP_LEASE,
    CAP_AUDIT_WRITE,
    CAP_AUDIT_CONTROL,
    CAP_SETFCAP,
    CAP_MAC_OVERRIDE,
    CAP_MAC_ADMIN,
    CAP_SYSLOG,
    CAP_WAKE_ALERM,
    CAP_BLOCK_SUSPEND,
    CAP_AUDIT_READ    
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(default)]
    pub uid: i64,
    #[serde(default)]
    pub gid: i64,
    #[serde(default, rename = "additionalGids")]
    pub additional_gids: Vec<i64>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Linux {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub namespaces: Vec<Namespace>,       
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "uidMappings")]
    pub uid_mappings: Vec<Mapping>,
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "gidMappings")]
    pub gid_mappings: Vec<Mapping>,
    #[serde(default)]
    pub devices: Vec<Device>,
    #[serde(default, rename = "cgroupsPath")]
    pub cgroup_path: String,
    #[serde(default)]
    pub resources: Option<Resources>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Namespace {
    #[serde(rename = "type")]
    pub typ: NamespaceType,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum NamespaceType {
    pid,
    network,
    mount,
    ipc,
    uts,
    user,
    cgroup,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mapping {
    #[serde(default, rename = "containerID")]
    pub container_id: u32,
    #[serde(default, rename = "hostID")]
    pub host_id: u32,
    #[serde(default)]
    pub size: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Device {
    #[serde(default, rename = "type")]
    pub typ: DeviceType,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub major: u64,
    #[serde(default)]
    pub minor: u64,
    #[serde(default, rename = "fileMode")]
    pub file_mode: u32,
    #[serde(default)]
    pub uid: u32,
    #[serde(default)]
    pub gid: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum DeviceType {
    b,
    c,
    u,
    p,
    a,    
}

impl Default for DeviceType {
    fn default() -> DeviceType {
        DeviceType::a    
    }    
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Resources {
    #[serde(default)]
    pub devices: Vec<DeviceCgroup>,
    #[serde(default)]
    pub memory: Option<MemoryCgroup>,
    #[serde(default)]
    pub cpu: Option<CpuCgroup>,
    #[serde(default, rename = "blockIO")]
    pub block_io: Option<BlockIo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "hugepageLimits")]
    pub hugepage_limits: Vec<HugepageLimit>,
    #[serde(default)]
    pub network: Option<Network>,
    #[serde(default)]
    pub pids: Option<Pids>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceCgroup {
    #[serde(default)]
    pub allow: bool,
    #[serde(default, rename = "type")]
    pub typ: String,
    #[serde(default)]
    pub major: i64,
    #[serde(default)]
    pub minor: i64,
    #[serde(default)]
    pub access: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MemoryCgroup {
    #[serde(default)]
    pub limit: i64,
    #[serde(default)]
    pub reservation: i64,
    #[serde(default)]
    pub swap: i64,
    #[serde(default)]
    pub kernel: i64,
    #[serde(default, rename = "kernelTCP")]
    pub kernel_tcp: i64,
    #[serde(default)]
    pub swappiness: u64,
    #[serde(default, rename = "disableOOMKiller")]
    pub disable_oom_killer: bool,    
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CpuCgroup {
    #[serde(default)]
    pub shares: u64,
    #[serde(default)]
    pub quota: i64,
    #[serde(default)]
    pub period: u64,
    pub realtime_runtime: i64,
    pub realtime_period: u64,
    #[serde(default)]
    pub cpus: String,
    #[serde(default)]
    pub mems: String,    
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockIo {
    #[serde(default)]
    pub weight: u16,
    #[serde(default, rename = "leafWeight")]
    pub leaf_weight: u16,
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "weightDevice")]
    pub weight_device: Vec<WeightDevice>,
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "throttleReadBpsDevice")]
    pub throttle_read_bps_device: Vec<BpsDevice>,
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "throttleWriteBpsDevice")]
    pub throttle_write_bps_device: Vec<BpsDevice>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WeightDevice {
    #[serde(default)]
    pub major: i64,
    #[serde(default)]
    pub minor: i64,
    #[serde(default)]
    pub weight: u16,
    #[serde(default, rename = "leafWeight")]
    pub leaf_weight: u16,    
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BpsDevice {
    #[serde(default)]
    pub major: i64,
    #[serde(default)]
    pub minor: i64,
    #[serde(default)]
    pub rate: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HugepageLimit {
    #[serde(default, rename = "pageSize")]
    pub page_size: String,
    #[serde(default)]
    pub limit: u64,    
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Network {
    #[serde(default, rename = "classID")]
    pub class_id: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub priorities: Vec<Priority>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Priority {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub priority: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pids {
    #[serde(default)]
    pub limit: i64,    
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Hooks {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prestart: Vec<Hook>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub poststart: Vec<Hook>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub poststop: Vec<Hook>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Hook {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub path: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i64>,
}

impl Spec {
    pub fn load() -> Result<Spec, Error> {
        let file = File::open("config.json")?;
        let buf_reader = BufReader::new(file);
        let spec: Spec = from_reader(buf_reader)?;
        Ok(spec)
    }        
}
