use std::collections::HashMap;

use indexmap::IndexMap;

use super::{DirectiveContext, DirectiveKind};
use crate::{
    articles::Article,
    directives::{Directive, date, destination},
    manifest::Manifest,
    resources::href,
};

#[derive(thiserror::Error, Debug)]
pub enum DirectiveError {
    #[error("Unable to find article with provided ID: {0}")]
    ArticleNotFound(String),

    #[error("The source for the provided binding is not valid: {0}")]
    InvalidBindingSource(String),

    #[error("Unable to find the provided resource: {0}")]
    StaticResourceNotFound(String),
}

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
    ) -> Result<String, DirectiveError> {
        match directive.kind {
            DirectiveKind::Binding => {
                let article = self
                    .articles
                    .get(id)
                    .ok_or_else(|| DirectiveError::ArticleNotFound(id.to_owned()))?;

                if context == DirectiveContext::Page {
                    process_page_binding(&directive.source, self.manifest, article)
                } else {
                    process_common_binding(&directive.source, self.manifest, article)
                }
            }
            DirectiveKind::Destination => {
                let referenced_article = self
                    .articles
                    .get(&directive.source)
                    .ok_or_else(|| DirectiveError::ArticleNotFound(directive.source.clone()))?;

                Ok(href::normalize(&destination::make_article_path(
                    self.manifest,
                    &referenced_article,
                    true,
                )))
            }
            DirectiveKind::StaticResource => self
                .resources
                .get(&directive.source)
                .map(|s| s.to_owned())
                .ok_or_else(|| DirectiveError::StaticResourceNotFound(directive.source.clone())),
        }
    }
}

fn process_common_binding(
    source: &str,
    manifest: &Manifest,
    article: &Article,
) -> Result<String, DirectiveError> {
    match source {
        "title" => Ok(article.title.clone()),
        "author" => Ok(article.author.clone()),
        "publishDate" => Ok(date::format_iso(&article.created)),
        "publishDisplayDate" => Ok(date::format_display(
            &article.created,
            &manifest.default_language,
        )),
        "updateDate" => Ok(date::format_iso(&article.updated)),
        "updateDisplayDate" => Ok(date::format_display(
            &article.updated,
            &manifest.default_language,
        )),
        "language" => Ok(manifest.default_language.clone()),
        "url" => Ok(href::normalize(&destination::make_article_path(
            manifest, article, true,
        ))),
        _ => Err(DirectiveError::InvalidBindingSource(source.to_owned())),
    }
}

fn process_page_binding(
    source: &str,
    manifest: &Manifest,
    article: &Article,
) -> Result<String, DirectiveError> {
    match source {
        "pageTitle" => {
            if article.default {
                Ok(manifest.title.clone())
            } else {
                Ok(format!("{} - {}", article.short_title, manifest.title))
            }
        }
        _ => process_common_binding(source, manifest, article),
    }
}
