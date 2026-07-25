use std::{collections::HashMap, fs, path::Path};

use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
enum ResourceValue {
    Map(HashMap<String, String>),
    String(String),
}

#[derive(thiserror::Error, Debug)]
pub enum ResourceMapLoadError {
    #[error("Found multiple resources with key '{0}' in resource map.")]
    DuplicateKeyError(String),

    #[error("Can't get path segments of URL '{0}'")]
    NonBaseUrlError(String),

    #[error(transparent)]
    ParseError(#[from] serde_json::Error),

    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

fn check_duplicate(
    resource_map: &HashMap<String, String>,
    key: &str,
) -> Result<(), ResourceMapLoadError> {
    if resource_map.contains_key(key) {
        Err(ResourceMapLoadError::DuplicateKeyError(key.to_owned()))
    } else {
        Ok(())
    }
}

pub fn load_resource_mapping(
    src: impl AsRef<Path>,
) -> Result<HashMap<String, String>, ResourceMapLoadError> {
    let file_contents = fs::read_to_string(&src)?;
    let record: HashMap<String, ResourceValue> = serde_json::from_str(&file_contents)?;

    let mut resource_map: HashMap<String, String> = HashMap::new();

    for (k, v) in &record {
        match v {
            ResourceValue::String(s) => {
                check_duplicate(&resource_map, k)?;
                resource_map.insert(k.clone(), s.clone());
            }
            ResourceValue::Map(inner_map) => {
                let url = Url::parse(k)?;

                for (key, value) in inner_map {
                    let mut full_path = url.clone();
                    {
                        let mut segments = full_path
                            .path_segments_mut()
                            .map_err(|_| ResourceMapLoadError::NonBaseUrlError(k.clone()))?;
                        segments.push(value);
                    }

                    check_duplicate(&resource_map, key)?;
                    resource_map.insert(key.clone(), full_path.to_string());
                }
            }
        }
    }

    Ok(resource_map)
}
