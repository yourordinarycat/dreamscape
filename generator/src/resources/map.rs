use std::{collections::HashMap, fs, path::Path};

use serde::Deserialize;
use url::Url;

#[derive(Deserialize, Debug)]
struct ResourceGroupOptions {
    base_url: Option<Url>,
}

#[derive(Deserialize, Debug)]
struct ResourceGroup {
    options: Option<ResourceGroupOptions>,
    resources: HashMap<String, String>,
}

#[derive(thiserror::Error, Debug)]
pub enum ResourceMapLoadError {
    #[error("Found multiple resources with key '{0}' in resource map.")]
    DuplicateKeyError(String),

    #[error(transparent)]
    ParseError(#[from] toml::de::Error),

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
    let record: HashMap<String, ResourceGroup> = toml::from_str(&file_contents)?;

    let mut resource_map: HashMap<String, String> = HashMap::new();

    for (_, group) in record.into_iter() {
        let base_url = group.options.map(|o| o.base_url).flatten();

        for (key, value) in group.resources.into_iter() {
            if let Some(ref url) = base_url {
                let full_path = url.join(&value)?;

                check_duplicate(&resource_map, &key)?;
                resource_map.insert(key, full_path.to_string());
            } else {
                check_duplicate(&resource_map, &key)?;
                resource_map.insert(key, value);
            }
        }
    }

    Ok(resource_map)
}
