use std::{fs, path::Path};

use icu::calendar::{Date, Iso, types::Month};
use indexmap::IndexMap;
use walkdir::WalkDir;

use crate::{
    articles::{
        Article,
        frontmatter::{FrontMatterParseError, extract_front_matter},
    },
    diagnostics::Diagnostics,
    manifest::Manifest,
};

#[derive(thiserror::Error, Debug)]
pub enum ArticleDiscoveryError {
    #[error("Found multiple articles with filename '{0}' in articles folder.")]
    Duplicate(String),

    #[error("Date '{0}' must use the ISO YYYY-MM-DD format.")]
    InvalidDateFormat(String),

    #[error(transparent)]
    InvalidFrontMatter(#[from] FrontMatterParseError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

fn parse_iso_date(date_str: &str) -> Result<Date<Iso>, ArticleDiscoveryError> {
    let make_err = || ArticleDiscoveryError::InvalidDateFormat(date_str.to_owned());

    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return Err(make_err());
    }

    let year: i32 = parts[0].parse().map_err(|_| make_err())?;
    let month_val: u8 = parts[1].parse().map_err(|_| make_err())?;
    let day: u8 = parts[2].parse().map_err(|_| make_err())?;

    Date::try_new(year.into(), Month::new(month_val), day, Iso).map_err(|_| make_err())
}

fn process_entry(
    manifest: &Manifest,
    vec: &mut Vec<Article>,
    options: &markdown::Options,
    entry: Result<walkdir::DirEntry, walkdir::Error>,
) -> Result<(), ArticleDiscoveryError> {
    let entry = entry.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    if entry.file_type().is_dir() {
        return Ok(());
    }

    let src_path = entry.path();
    let content = fs::read_to_string(&src_path)?;

    let html_output = markdown::to_html_with_options(&content, &options)
        .expect("Failed to render Markdown to HTML");

    let metadata = extract_front_matter(&content)?;
    let id = entry
        .path()
        .file_stem()
        .expect("File stem should not be None.")
        .to_str()
        .expect("Failed to convert string to UTF-8.")
        .to_string();

    let created = parse_iso_date(&metadata.created)?;
    let updated = if let Some(updated_at) = metadata.updated {
        parse_iso_date(&updated_at)?
    } else {
        created
    };

    let short_title = metadata
        .short_title
        .unwrap_or_else(|| metadata.title.clone());
    let default = id == "index";

    vec.push(Article {
        id,
        title: metadata.title,
        short_title,
        author: metadata.author.unwrap_or(manifest.author.clone()),
        created,
        updated,
        content: html_output,
        layout: metadata.layout.unwrap_or(manifest.default_layout.clone()),
        default,
    });

    Ok(())
}

pub fn find_all(
    src: impl AsRef<Path>,
    manifest: &Manifest,
) -> (IndexMap<String, Article>, Diagnostics) {
    let mut diagnostics = Diagnostics::default();
    let mut vec: Vec<Article> = Vec::new();
    let src = src.as_ref();

    let markdown_parse_options = markdown::Options {
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
        match process_entry(manifest, &mut vec, &markdown_parse_options, entry) {
            Ok(_) => (),
            Err(err) => {
                diagnostics.errors.push(err.into());
            }
        };
    }

    let mut map: IndexMap<String, Article> = IndexMap::new();
    vec.sort_by(|a, b| {
        let is_a_default = a.default;
        let is_b_default = b.default;

        match (is_a_default, is_b_default) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            (false, false) => b.created.cmp(&a.created),
        }
    });

    for elm in vec {
        if !map.contains_key(&elm.id) {
            map.insert(elm.id.clone(), elm);
        } else {
            diagnostics
                .errors
                .push(ArticleDiscoveryError::Duplicate(elm.id.clone()).into());
        }
    }
    (map, diagnostics)
}
