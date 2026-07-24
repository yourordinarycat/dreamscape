use crate::manifest::Manifest;
use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Deserialize, Debug)]
pub struct FrontMatter {
    pub title: String,
    pub created: String,
    pub updated: Option<String>,

    pub author: Option<String>,
    pub short_title: Option<String>,
    pub layout: Option<String>,
}

#[derive(Debug)]
pub struct Post {
    pub title: String,
    pub short_title: String,
    pub author: String,

    pub created: String,
    pub updated: String,
    pub content: String,
    pub layout: String,
    pub default: bool,
}

#[derive(Error, Debug)]
pub enum FrontMatterParseError {
    #[error("The file's front matter is not formatted properly: {0}")]
    ParseError(#[from] yaml_serde::Error),
    #[error("The file doesn't have a valid front matter: {0}")]
    ErrorMessage(String),
}

fn extract_front_matter(content: &str) -> Result<FrontMatter, FrontMatterParseError> {
    if !content.starts_with("---") {
        return Err(FrontMatterParseError::ErrorMessage(
            "File does not have a front matter.".to_string(),
        ));
    }

    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() >= 3 {
        let yaml_block = parts[1];
        yaml_serde::from_str::<FrontMatter>(yaml_block).map_err(FrontMatterParseError::ParseError)
    } else {
        Err(FrontMatterParseError::ErrorMessage(
            "Front matter does not have a closing tag.".to_string(),
        ))
    }
}

pub fn get_posts(
    src: impl AsRef<Path>,
    manifest: &Manifest,
) -> Result<HashMap<String, Post>, Box<dyn std::error::Error>> {
    let mut map: HashMap<String, Post> = HashMap::new();

    let src = src.as_ref();

    let custom_options = markdown::Options {
        parse: markdown::ParseOptions {
            constructs: markdown::Constructs {
                frontmatter: true,
                ..markdown::Constructs::gfm()
            },
            ..markdown::ParseOptions::gfm()
        },
        ..markdown::Options::gfm()
    };

    for entry in WalkDir::new(src) {
        let entry = entry.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        if entry.file_type().is_dir() {
            continue;
        }

        let src_path = entry.path();
        let content = fs::read_to_string(&src_path)?;

        let html_output = markdown::to_html_with_options(&content, &custom_options)
            .expect("Failed to render Markdown to HTML");

        let metadata = extract_front_matter(&content)?;
        let id = entry
            .path()
            .file_stem()
            .expect("File stem should not be None.")
            .to_str()
            .expect("Failed to convert string to UTF-8.")
            .to_string();

        let short_title = metadata
            .short_title
            .unwrap_or_else(|| metadata.title.clone());
        let default = id == "index";

        map.insert(id, Post {
            title: metadata.title,
            short_title,
            author: metadata.author.unwrap_or(manifest.author.clone()),
            updated: metadata.updated.unwrap_or(metadata.created.clone()),
            created: metadata.created,
            content: html_output,
            layout: metadata.layout.unwrap_or(manifest.default_layout.clone()),
            default,
        });
    }

    Ok(map)
}
