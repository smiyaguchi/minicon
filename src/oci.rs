#[derive(Deserialize, Debug)]
pub enum NamespaceType {
    pid,
    network,
    mount,
    ipc,
    uts,
    user,
    cgroup,  
}

#[derive(Deserialize, Debug)]
pub struct Namespace {
    #[serde(rename = "type")]
    pub typ: NamespaceType,
    #[serde(default)]
    pub path: String, 
}

#[derive(Deserialize, Debug)]
pub struct Linux {
    pub namespaces: Vec<Namespace>,
}

#[derive(Deserialize, Debug)]
pub struct Spec {
    pub linux: Linux,
}
