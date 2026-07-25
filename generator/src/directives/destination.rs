use std::path::{Path, PathBuf};

use super::date;
use crate::{articles::Article, manifest::Manifest};

fn make_path(article: &Article, default_is_empty: bool) -> PathBuf {
    if article.default {
        if default_is_empty {
            PathBuf::new()
        } else {
            Path::new("index.html").to_path_buf()
        }
    } else {
        let mut id = article.id.to_string();
        id.push_str(".html");

        let mut buf = PathBuf::new();
        buf.push(date::format_iso(&article.created).replace('-', "/"));
        buf.push(id);

        buf
    }
}

pub fn make_article_path(
    manifest: &Manifest,
    article: &Article,
    default_is_empty: bool,
) -> PathBuf {
    let article_path = make_path(&article, default_is_empty);

    if let Some(base_url) = &manifest.base_path {
        let base_path = Path::new(base_url);
        base_path.join(article_path)
    } else {
        article_path
    }
}
