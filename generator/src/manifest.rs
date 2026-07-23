use serde::Deserialize;
use std::fs;
use std::convert::AsRef;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct Manifest {
    pub title: String,
    pub author: String,
    pub default_language: String,
    pub default_layout: String,
    pub base_path: Option<String>,
}

pub fn load_manifest(src: impl AsRef<Path>) -> Result<Manifest, Box<dyn std::error::Error>> {
    let file_contents = fs::read_to_string(&src)?;
    let manifest: Manifest = serde_json::from_str(&file_contents)?;

    Ok(manifest)
}