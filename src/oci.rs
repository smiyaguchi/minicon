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
pub struct Linux {
    pub namespaces: Vec<Namespace>,
    #[serde(rename = "uidMappings")]
    pub uid_mappings: Vec<IDMapping>,
    #[serde(rename = "gidMappings")]
    pub gid_mappings: Vec<IDMapping>,
}

#[derive(Deserialize, Debug)]
pub struct Spec {
    #[serde(rename = "ociVersion")]
    pub oci_version: String,
    pub root: Root,
    pub linux: Linux,
}
