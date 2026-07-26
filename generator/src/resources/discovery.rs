use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::{diagnostics::Diagnostics, path_ext, resources::map::ResourceMapLoadError};

use super::{href, map};

#[derive(thiserror::Error, Debug)]
pub enum ResourceDiscoveryError {
    #[error("Found multiple resources with key '{0}' in resources folder.")]
    DuplicateKeyError(String),

    #[error(transparent)]
    ResourceMapLoadError(#[from] ResourceMapLoadError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

fn check_duplicate(
    resource_map: &HashMap<String, String>,
    key: &str,
) -> Result<(), ResourceDiscoveryError> {
    if resource_map.contains_key(key) {
        Err(ResourceDiscoveryError::DuplicateKeyError(key.to_owned()))
    } else {
        Ok(())
    }
}

fn extend_checked(
    resource_map: &mut HashMap<String, String>,
    with: &HashMap<String, String>,
) -> Result<(), ResourceDiscoveryError> {
    for (key, value) in with {
        check_duplicate(resource_map, key)?;
        resource_map.insert(key.clone(), value.clone());
    }

    Ok(())
}

fn process_entry(
    src: &Path,
    dst: &Path,
    base_dst: &Path,
    resource_map: &mut HashMap<String, String>,
    entry: Result<walkdir::DirEntry, walkdir::Error>,
) -> Result<(), ResourceDiscoveryError> {
    let entry = entry.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let src_path = entry.path();

    // Calculate the relative path from the source root
    let relative_path = src_path
        .strip_prefix(src)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Combine relative path with the destination root
    let dst_path = dst.join(relative_path);

    // Replicate the folder structure
    if src_path.is_dir() {
        fs::create_dir_all(&dst_path)?;
        return Ok(());
    }

    let name = path_ext::name_to_str(src_path.file_name())?;

    // If the file is a JSON resource map, process it instead of copying
    if name == "map.json" {
        let mapping = map::load_resource_mapping(src_path)?;

        extend_checked(resource_map, &mapping)?;
        return Ok(());
    }

    fs::copy(src_path, &dst_path)?;

    // Add resource to map
    let key = path_ext::name_to_str(src_path.file_stem())?;

    let target = dst_path
        .strip_prefix(base_dst)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    check_duplicate(&resource_map, key)?;
    resource_map.insert(key.to_owned(), href::normalize(target));

    Ok(())
}

pub fn find_all(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    base_dst: impl AsRef<Path>,
) -> (HashMap<String, String>, Diagnostics) {
    let mut diagnostics = Diagnostics::default();
    let mut resource_map: HashMap<String, String> = HashMap::new();

    let src = src.as_ref();
    let dst = dst.as_ref();
    let base_dst = base_dst.as_ref();

    for entry in WalkDir::new(src) {
        match process_entry(src, dst, base_dst, &mut resource_map, entry) {
            Ok(_) => (),
            Err(err) => {
                diagnostics.errors.push(err.into());
            }
        };
    }

    (resource_map, diagnostics)
}
