use dom_query::Document;
use std::{collections::HashMap, fs, path::Path};
use walkdir::WalkDir;

#[derive(Debug, PartialEq, Eq)]
pub enum DirectiveKind {
    Binding,
    Destination,
    StaticResource,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TargetKind {
    Attribute,
    Property,
}

#[derive(Debug)]
pub struct Directive {
    pub kind: DirectiveKind,
    pub source: String,
    pub target: String,
    pub target_kind: TargetKind,
}

#[derive(Debug)]
pub struct Layout {
    pub name: String,
    pub content: String,
    pub directives: HashMap<u8, Vec<Directive>>,
}

fn remove_first_and_last(s: &str) -> &str {
    let mut chars = s.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}

fn parse_directive(key: &str, value: &str) -> Option<Directive> {
    if !value.starts_with("{") || !value.ends_with("}") {
        return None;
    }

    let content: Vec<&str> = remove_first_and_last(value).split(" ").collect();
    if content.len() > 2 {
        return None;
    }

    let directive_kind: Option<DirectiveKind> = match content[0] {
        "Binding" => Some(DirectiveKind::Binding),
        "Destination" => Some(DirectiveKind::Destination),
        "StaticResource" => Some(DirectiveKind::StaticResource),
        _ => None,
    };

    if directive_kind.is_none() {
        return None;
    }

    let target_kind: TargetKind = if key.starts_with("attr.") {
        TargetKind::Attribute
    } else {
        TargetKind::Property
    };

    let source = if content.len() == 1 { "" } else { content[1] };
    let target = if target_kind == TargetKind::Attribute {
        key.strip_prefix("attr.").unwrap()
    } else {
        key
    };

    Some(Directive {
        kind: directive_kind.unwrap(),
        source: source.to_string(),
        target: target.to_string(),
        target_kind,
    })
}

fn process_layout(name: &str, content: &str) -> Result<Layout, Box<dyn std::error::Error>> {
    let document = Document::from(content);
    let mut directive_map: HashMap<u8, Vec<Directive>> = HashMap::new();
    let mut curr_cid: u8 = 0;

    let nodes = document.select("*").iter();

    for node in nodes {
        let mut directives: Vec<Directive> = Vec::new();

        for attr in node.attrs() {
            let name = &attr.name.local;
            let value = &attr.value;
            
            let parsed_directive = parse_directive(name, value);
    
            if let Some(directive) = parsed_directive {
                directives.push(directive);
                node.remove_attr(name);
            }
        }

        if directives.len() > 0 {
            directive_map.insert(curr_cid, directives);

            node.set_attr("data-blg-connection-id", &curr_cid.to_string());
            curr_cid += 1;
        }
    }

    Ok(Layout {
        name: name.to_string(),
        content: document.html().to_string(),
        directives: directive_map,
    })
}

pub fn get_layouts(src: impl AsRef<Path>) -> Result<Vec<Layout>, Box<dyn std::error::Error>> {
    let mut vec: Vec<Layout> = Vec::new();

    let src = src.as_ref();

    for entry in WalkDir::new(src) {
        let entry = entry.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        if entry.file_type().is_dir() {
            continue;
        }

        let src_path = entry.path();
        let content = fs::read_to_string(&src_path)?;

        let name = entry
            .path()
            .file_stem()
            .expect("File stem should be present.")
            .to_str()
            .expect("Failed to convert file stem to string.");

        let layout = process_layout(name, &content)?;
        vec.push(layout);
    }

    Ok(vec)
}
