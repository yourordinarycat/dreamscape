pub mod discovery;
mod frontmatter;
pub mod writer;

use icu::calendar::{Date, Iso};

#[derive(Debug)]
pub struct Article {
    pub id: String,
    pub title: String,
    pub short_title: String,
    pub author: String,

    pub created: Date<Iso>,
    pub updated: Date<Iso>,
    pub content: String,
    pub layout: String,
    pub default: bool,
}
