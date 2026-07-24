use std::{collections::HashMap, fs, path::Path};

use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
enum ResourceValue {
    Map(HashMap<String, String>),
    String(String),
}

pub fn load_resource_mapping(
    src: impl AsRef<Path>,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let file_contents = fs::read_to_string(&src)?;
    let record: HashMap<String, ResourceValue> = serde_json::from_str(&file_contents)?;

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
