use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use super::{href, map};

pub fn find_all(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    base_dst: impl AsRef<Path>,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut resource_map: HashMap<String, String> = HashMap::new();

    let src = src.as_ref();
    let dst = dst.as_ref();
    let base_dst = base_dst.as_ref();

    for entry in WalkDir::new(src) {
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
            continue;
        }

        let name = src_path
            .file_name()
            .expect("File name should never be empty.");

        // If the file is a JSON resource map, process it instead of copying
        if name == "map.json" {
            let mapping = map::load_resource_mapping(src_path)?;

            // TODO: Duplicate checks
            resource_map.extend(mapping);
            continue;
        }

        fs::copy(src_path, &dst_path)?;

        // Add resource to map
        let key = src_path
            .file_stem()
            .expect("File stem should never be empty.");

        let target = dst_path.strip_prefix(base_dst)?;

        // TODO: Duplicate check
        resource_map.insert(key.to_str().unwrap().to_owned(), href::normalize(target));
    }

    Ok(resource_map)
}
