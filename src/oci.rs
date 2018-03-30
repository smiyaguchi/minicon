use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct State {
    #[serde(rename = "ociVersion")]
    pub oci_version: String,
    pub id: String,
    pub status: String,
    pub pid: i32,
    pub bundle: String,
    pub annotations: HashMap<String, String>,    
}

#[derive(Deserialize, Debug)]
pub struct ConsoleSize {
    pub height: u64,
    pub width: u64  
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum RlimitType {
    RLIMIT_CPU,
    RLIMIT_FSIZE,
    RLIMIT_DATA,
    RLIMIT_STACK,
    RLIMIT_CORE,
    RLIMIT_RSS,
    RLIMIT_NPROC,
    RLIMIT_NOFILE,
    RLIMIT_MEMLOCK,
    RLIMIT_AS,
    RLIMIT_LOCKS,
    RLIMIT_SIGPENDING,
    RLIMIT_MSGQUEUE,
    RLIMIT_NICE,
    RLIMIT_RTPRIO,
    RLIMIT_RTTIME,
}

#[derive(Deserialize, Debug)]
pub struct Rlimit {
    #[serde(rename = "type")]
    pub typ: RlimitType,
    pub soft: u64,
    pub hard: u64,
}

#[derive(Deserialize, Debug)]
pub struct Process {
    pub terminal: bool,
    #[serde(rename = "consoleSize")]
    pub console_size: ConsoleSize,
    pub cwd: String,
    pub env: Vec<String>,
    pub args: Vec<String>, 
    pub rlimits: Vec<Rlimit>,
    #[serde(rename = "oomScoreAdj")]
    pub oom_score_adj: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct Root {
    pub path: String,
    #[serde(rename = "readonly")]
    pub read_only: bool,  
}

#[derive(Deserialize, Debug)]
pub struct Namespace {
    #[serde(rename = "type")]
    pub typ: String,
    #[serde(default)]
    pub path: String, 
}

#[derive(Deserialize, Debug)]
pub struct IDMapping {
    #[serde(rename = "hostID")] 
    pub host_id: u32,
    #[serde(rename = "containerID")] 
    pub container_id: u32,
    pub size: u32,
}

#[derive(Deserialize, Debug)]
pub struct Device {
    pub allow: bool,
    #[serde(rename = "type")]
    pub typ: String,
    pub major: u64,
    pub minor: u64,
    pub access: String,
}

#[derive(Deserialize, Debug)]
pub struct Memory {
    pub limit: i64,
    pub reservation: i64,
    pub swap: i64,
    pub kernel: i64,
    #[serde(rename = "kernelTCP")]
    pub kernel_tcp: i64,
    pub swappiness: u64,
    #[serde(rename = "disableOOMKiller")]
    pub disable_oom_killer: bool,
}

#[derive(Deserialize, Debug)]
pub struct Cpu {
    pub shares: u64,
    pub quota: i64,
    pub period: u64,
    #[serde(rename = "realtimeRuntime")]
    pub realtime_runtime: i64,
    #[serde(rename = "realtimePeriod")]
    pub realtime_period: u64,
    pub cpus: String,
    pub mems: String,
}

#[derive(Deserialize, Debug)]
pub struct WeightDevice {
    pub major: i64,
    pub minor: i64,
    pub weight: u16,
    #[serde(rename = "leafWeight")]
    pub leaf_weight: u16,
}

#[derive(Deserialize, Debug)]
pub struct ThrottleDevice {
    pub major: i64,
    pub minor: i64,
    pub rate: u64,
}

#[derive(Deserialize, Debug)]
pub struct BlockIO {
    pub weight: u16,
    pub leaf_weight: u16,
    #[serde(rename = "weightDevice")]
    pub weight_device: Vec<WeightDevice>,
    #[serde(rename = "throttleReadBpsDevice")]
    pub throttle_read_bps_device: Vec<ThrottleDevice>,
    #[serde(rename = "throttleWriteBpsDevice")]
    pub throttle_write_bps_device: Vec<ThrottleDevice>,
    #[serde(rename = "throttleReadIOPSDevice")]
    pub throttle_read_iops_device: Vec<ThrottleDevice>,
    #[serde(rename = "throttleWriteIOPSDevice")]
    pub throttle_write_iops_device: Vec<ThrottleDevice>,
}

#[derive(Deserialize, Debug)]
pub struct HugepageLimit {
    #[serde(rename = "pageSize")]
    pub page_size: String,
    pub limit: u64,
}

#[derive(Deserialize, Debug)]
pub struct Priority {
    pub name: String,
    pub priority: u32,
} 

#[derive(Deserialize, Debug)]
pub struct Network {
    #[serde(rename = "classID")]
    pub class_id: u32,
    pub priorities: Vec<Priority>,
}

#[derive(Deserialize, Debug)]
pub struct Pids {
    pub limit: i64,
}

#[derive(Deserialize, Debug)]
pub struct Resources {
    pub devices: Vec<Device>,
    pub memory: Memory,
    pub cpu: Cpu,
    #[serde(rename = "blockIO")]
    pub block_io: BlockIO,
    #[serde(rename = "hugepageLimits")]
    pub hugepage_limits: Vec<HugepageLimit>,
    pub network: Network,
    pub pids: Pids,
}

#[derive(Deserialize, Debug)]
pub struct Hca {
    #[serde(rename = "hcaHandles")]
    pub hca_handles: u32,
    #[serde(rename = "hcaObjects")]
    pub hca_objects: u32,
}

#[derive(Deserialize, Debug)]
pub struct Rdma {
    pub device: HashMap<String, Hca>, 
}

#[derive(Deserialize, Debug)]
pub struct Linux {
    pub namespaces: Vec<Namespace>,
    #[serde(rename = "uidMappings")]
    pub uid_mappings: Vec<IDMapping>,
    #[serde(rename = "gidMappings")]
    pub gid_mappings: Vec<IDMapping>,
    #[serde(rename = "cgroupPath")]
    pub cgroup_path: String,

}

#[derive(Deserialize, Debug)]
pub struct Spec {
    #[serde(rename = "ociVersion")]
    pub oci_version: String,
    pub root: Root,
    pub process: Process,
    pub hostname: String,
    pub linux: Linux,
    pub annotations: HashMap<String, String>,
}
