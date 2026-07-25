use std::path::{PathBuf};

use super::date;
use crate::{articles::Article, manifest::Manifest};

pub fn make_article_path(
    manifest: &Manifest,
    article: &Article,
    default_is_empty: bool,
) -> PathBuf {
    let mut buf = PathBuf::new();

    if let Some(base_url) = &manifest.base_path {
        buf.push(base_url);
    } else {
        buf.push("/");
    };

    if article.default {
        if !default_is_empty {
            buf.push("index.html");
        }
    } else {
        let mut id = article.id.to_string();
        id.push_str(".html");

        buf.push(date::format_iso(&article.created).replace('-', "/"));
        buf.push(id);
    }
    buf
}
