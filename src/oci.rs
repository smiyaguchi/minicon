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
pub struct Linux {
    pub namespaces: Vec<Namespace>,
}

#[derive(Deserialize, Debug)]
pub struct Spec {
    #[serde(rename = "ociVersion")]
    pub oci_version: String,
    pub root: Root,
    pub linux: Linux,
}
