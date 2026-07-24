use std::path::Path;

pub fn normalize(path: &Path) -> String {
    let mut href = path
        .to_str()
        .expect("Something went wrong converting the path to a string.")
        .replace('\\', "/");

    if !href.starts_with('/') {
        href.insert(0, '/')
    }

    href
}
