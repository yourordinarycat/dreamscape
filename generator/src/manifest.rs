use serde::Deserialize;
use std::convert::AsRef;
use std::fs;
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum ManifestError {
    #[error(transparent)]
    ParseError(#[from] serde_json::Error),

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

pub fn load_manifest(src: impl AsRef<Path>) -> Result<Manifest, ManifestError> {
    let file_contents = fs::read_to_string(&src)?;
    let manifest: Manifest = serde_json::from_str(&file_contents)?;

    Ok(manifest)
}
