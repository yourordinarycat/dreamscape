use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, Debug)]
pub struct FrontMatter {
    pub title: String,
    pub created: String,
    pub updated: Option<String>,

    pub author: Option<String>,
    pub short_title: Option<String>,
    pub layout: Option<String>,
}

#[derive(Error, Debug)]
pub enum FrontMatterParseError {
    #[error("The file's front matter is not formatted properly")]
    FormatError(#[from] yaml_serde::Error),
    #[error("The file doesn't have a valid front matter: {0}")]
    ParseError(String),
}

pub fn extract_front_matter(content: &str) -> Result<FrontMatter, FrontMatterParseError> {
    if !content.starts_with("---") {
        return Err(FrontMatterParseError::ParseError(
            "File does not have a front matter.".to_string(),
        ));
    }

    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() >= 3 {
        let yaml_block = parts[1];
        yaml_serde::from_str::<FrontMatter>(yaml_block).map_err(FrontMatterParseError::FormatError)
    } else {
        Err(FrontMatterParseError::ParseError(
            "Front matter does not have a closing tag.".to_string(),
        ))
    }
}
