use std::{fs, path::Path};

use icu::calendar::{Date, Iso, types::Month};
use indexmap::IndexMap;
use walkdir::WalkDir;

use crate::{manifest::Manifest, articles::{Article, frontmatter::extract_front_matter}};

fn parse_iso_date(date_str: &str) -> Result<Date<Iso>, &'static str> {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return Err("Invalid format, expected YYYY-MM-DD");
    }

    let year: i32 = parts[0].parse().map_err(|_| "Invalid year.")?;
    let month_val: u8 = parts[1].parse().map_err(|_| "Invalid month.")?;
    let day: u8 = parts[2].parse().map_err(|_| "Invalid day.")?;

    Date::try_new(year.into(), Month::new(month_val), day, Iso)
        .map_err(|_| "Out of range date values")
}

pub fn find_all(
    src: impl AsRef<Path>,
    manifest: &Manifest,
) -> Result<IndexMap<String, Article>, Box<dyn std::error::Error>> {
    let mut vec: Vec<Article> = Vec::new();
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
        map.insert(elm.id.clone(), elm);
    }
    Ok(map)
}
