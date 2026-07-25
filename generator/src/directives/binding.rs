use std::collections::HashMap;

use indexmap::IndexMap;

use super::DirectiveContext;
use crate::{
    articles::Article,
    directives::{Directive, date, destination},
    manifest::Manifest,
    resources::href,
};

pub struct BindingContext<'a> {
    pub manifest: &'a Manifest,
    pub articles: &'a IndexMap<String, Article>,
    pub resources: &'a HashMap<String, String>,
}

impl<'a> BindingContext<'a> {
    pub fn get(
        &self,
        id: &str,
        directive: &Directive,
        context: DirectiveContext,
    ) -> Option<String> {
        match directive.kind {
            super::DirectiveKind::Binding => {
                let article = self
                    .articles
                    .get(id)
                    .expect("Unable to find article with provided ID");

                if context == DirectiveContext::Page {
                    process_page_binding(&directive.source, self.manifest, article)
                } else {
                    process_common_binding(&directive.source, self.manifest, article)
                }
            }
            super::DirectiveKind::Destination => {
                let referenced_article = self
                    .articles
                    .get(&directive.source)
                    .expect("Destination references an article that doesn't exist.");

                destination::make_article_path(self.manifest, &referenced_article, true)
                    .to_str()
                    .map(String::from)
            }
            super::DirectiveKind::StaticResource => self
                .resources
                .get(&directive.source)
                .map(|s| s.to_owned()),
        }
    }
}

fn process_common_binding(source: &str, manifest: &Manifest, article: &Article) -> Option<String> {
    match source {
        "title" => Some(article.title.clone()),
        "author" => Some(article.author.clone()),
        "publishDate" => Some(date::format_iso(&article.created)),
        "publishDisplayDate" => Some(date::format_display(
            &article.created,
            &manifest.default_language,
        )),
        "updateDate" => Some(date::format_iso(&article.updated)),
        "updateDisplayDate" => Some(date::format_display(
            &article.updated,
            &manifest.default_language,
        )),
        "language" => Some(manifest.default_language.clone()),
        "url" => Some(href::normalize(&destination::make_article_path(
            manifest, article, true,
        ))),
        _ => None,
    }
}

fn process_page_binding(source: &str, manifest: &Manifest, article: &Article) -> Option<String> {
    match source {
        "pageTitle" => {
            if article.default {
                Some(manifest.title.clone())
            } else {
                Some(format!("{} - {}", article.short_title, manifest.title))
            }
        }
        _ => process_common_binding(source, manifest, article),
    }
}
