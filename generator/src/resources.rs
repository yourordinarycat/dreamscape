use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use url::Url;
use walkdir::WalkDir;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
enum ResourceValue {
    Map(HashMap<String, String>),
    String(String),
}

fn load_resource_mapping(
    src: impl AsRef<Path>,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let file_contents = fs::read_to_string(&src)?;
    let record: HashMap<String, ResourceValue> = serde_json::from_str(&file_contents)?;
    println!("Loading resource map: {:#?}", record);

    let mut resource_map: HashMap<String, String> = HashMap::new();

    for (k, v) in &record {
        // TODO: Duplicate checks
        match v {
            ResourceValue::String(s) => {
                resource_map.insert(k.clone(), s.clone());
            }
            ResourceValue::Map(inner_map) => {
                let url = Url::parse(k).unwrap();

                for (key, value) in inner_map {
                    let mut full_path = url.clone();
                    {
                        let mut segments = full_path.path_segments_mut().unwrap();
                        segments.push(value);
                    }

                    resource_map.insert(key.clone(), full_path.to_string());
                }
            }
        }
    }

    Ok(resource_map)
}

fn normalize_to_href(path: &Path) -> String {
    let mut href = path
        .to_str()
        .expect("Something went wrong converting the path to a string.")
        .replace('\\', "/");

    if !href.starts_with('/')  {
        href = format!("/{}", href);
    }

    href
}

pub fn create_resource_map(
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
            let mapping = load_resource_mapping(src_path)?;

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
        resource_map.insert(
            key.to_str().unwrap().to_owned(),
            normalize_to_href(target),
        );
    }

    Ok(resource_map)
}
