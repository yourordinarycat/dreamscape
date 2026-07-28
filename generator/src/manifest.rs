use serde::Deserialize;
use std::convert::AsRef;
use std::fs;
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum ManifestError {
    #[error(transparent)]
    ParseError(#[from] toml::de::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Deserialize, Debug)]
pub struct Manifest {
    pub title: String,
    pub author: String,
    pub default_language: String,
    pub default_layout: String,
    pub base_path: Option<String>,
}

#[derive(Deserialize)]
struct ManifestInfo {
    manifest: Manifest,
}

pub fn load_manifest(src: impl AsRef<Path>) -> Result<Manifest, ManifestError> {
    let file_contents = fs::read_to_string(&src)?;
    let manifest_info: ManifestInfo = toml::from_str(&file_contents)?;

    Ok(manifest_info.manifest)
}
