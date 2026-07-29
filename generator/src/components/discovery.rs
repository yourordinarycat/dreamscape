use dom_query::Document;
use std::{collections::HashMap, fs, path::Path};
use walkdir::WalkDir;

use crate::{
    components::Component,
    diagnostics::Diagnostics,
    directives::{self, Directive, connection::CONNECTION_ID_ATTR},
    path_ext,
};

#[derive(thiserror::Error, Debug)]
pub enum ComponentDiscoveryError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

fn process_component(content: &str) -> Component {
    let document = Document::from(content);
    let mut directive_map: HashMap<u8, Vec<Directive>> = HashMap::new();
    let mut curr_cid: u8 = 0;

    let nodes = document.select("*").iter();

    for node in nodes {
        let directives: Vec<_> = directives::html::from_node(&node).collect();

        if directives.len() > 0 {
            directive_map.insert(curr_cid, directives);

            node.set_attr(CONNECTION_ID_ATTR, &curr_cid.to_string());
            curr_cid += 1;
        }
    }

    Component {
        content: document.html().to_string(),
        directives: directive_map,
    }
}

fn process_entry(
    map: &mut HashMap<String, Component>,
    entry: Result<walkdir::DirEntry, walkdir::Error>,
) -> Result<(), ComponentDiscoveryError> {
    let entry = entry.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    if entry.file_type().is_dir() {
        return Ok(());
    }

    let src_path = entry.path();
    let content = fs::read_to_string(&src_path)?;

    let name = path_ext::name_to_str(src_path.file_stem())?;

    let layout = process_component(&content);
    map.insert(name.to_owned(), layout);

    Ok(())
}

pub fn find_all(src: impl AsRef<Path>) -> (HashMap<String, Component>, Diagnostics) {
    let mut diagnostics = Diagnostics::default();
    let mut map: HashMap<String, Component> = HashMap::new();

    let src = src.as_ref();

    for entry in WalkDir::new(src) {
        match process_entry(&mut map, entry) {
            Ok(_) => (),
            Err(err) => {
                diagnostics.errors.push(err.into());
            }
        };
    }

    (map, diagnostics)
}
