use dom_query::Document;
use indexmap::IndexMap;
use std::{collections::HashMap, fs, path::Path};

use crate::{
    articles::Article,
    components::{
        Component, article::ArticleElement, article_list::ArticleListElement,
        article_preview::ArticlePreviewElement,
    },
    diagnostics::Diagnostics,
    directives::{
        DirectiveContext,
        binding::BindingContext,
        connection::{CONNECTION_ID_ATTR, CONNECTION_ID_ATTR_SELECTOR},
        context::{
            BLG_ARTICLE_LIST_TAG, BLG_ARTICLE_TAG, BLG_NEXT_ARTICLE_TAG, BLG_PREVIOUS_ARTICLE_TAG,
        },
        destination, html,
    },
    manifest::Manifest,
};

#[derive(thiserror::Error, Debug)]
pub enum WriterError {
    #[error("Unable to find layout with provided ID: {0}")]
    LayoutNotFound(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

fn process_article(
    id: &str,
    context: &BindingContext,
    components: &HashMap<String, Component>,
    layout: &Component,
) -> (Option<String>, Diagnostics) {
    let mut diagnostics = Diagnostics::default();

    let document = Document::from(&*layout.content);

    let article_nodes = document.select(BLG_ARTICLE_TAG).iter();
    for node in article_nodes {
        let mut article_element = ArticleElement::new(&node);
        diagnostics.merge(article_element.inflate(id, context));
    }

    let prev_idx = context.articles.get_index_of(id).unwrap() + 1;
    let prev_article_nodes = document.select(BLG_PREVIOUS_ARTICLE_TAG);

    if let Some((_, previous)) = context.articles.get_index(prev_idx)
        && !previous.default
    {
        for node in prev_article_nodes.iter() {
            let mut article_preview_element = ArticlePreviewElement::new(&node);
            diagnostics.merge(article_preview_element.inflate(&previous.id, context));
        }
    } else {
        prev_article_nodes.remove();
    }

    let next_idx_opt = context.articles.get_index_of(id).unwrap().checked_sub(1);
    let next_article_nodes = document.select(BLG_NEXT_ARTICLE_TAG);

    if let Some(next_idx) = next_idx_opt
        && let Some((_, next)) = context.articles.get_index(next_idx)
        && !next.default
    {
        for node in next_article_nodes.iter() {
            let mut article_preview_element = ArticlePreviewElement::new(&node);
            diagnostics.merge(article_preview_element.inflate(&next.id, context));
        }
    } else {
        next_article_nodes.remove();
    }

    let article_list_nodes = document.select(BLG_ARTICLE_LIST_TAG).iter();
    for node in article_list_nodes {
        let mut article_list_element = ArticleListElement::new(&node);
        diagnostics.merge(article_list_element.inflate(context));
    }

    // Process custom components
    for (tag, component) in components {
        let component_nodes = document.select(tag).iter();
        for node in component_nodes {
            node.set_html(component.content.clone());
            let children = node.select(CONNECTION_ID_ATTR_SELECTOR).iter();

            for child in children {
                let cid: u8 = child.attr(CONNECTION_ID_ATTR).unwrap().parse().unwrap();
                child.remove_attr(CONNECTION_ID_ATTR);

                let directives = component.directives.get(&cid).unwrap();

                for directive in directives {
                    if let Some(value) = node.attr(&directive.source) {
                        html::apply(&child, directive, &value);
                    }
                }
            }

            if !node.children().first().is("template") {
                node.replace_with_selection(&node.children());
            }
        }
    }

    // Process remaining elements with connection IDs
    let nodes = document.select(CONNECTION_ID_ATTR_SELECTOR);
    let errs = html::apply_all(
        nodes.iter(),
        id,
        &layout.directives,
        context,
        DirectiveContext::Page,
    );

    diagnostics
        .errors
        .extend(errs.into_iter().map(|e| e.into()));

    if diagnostics.failed() {
        (None, diagnostics)
    } else {
        (Some(document.html().to_string()), diagnostics)
    }
}

pub fn write_to(
    dst: impl AsRef<Path>,
    manifest: &Manifest,
    articles: &IndexMap<String, Article>,
    components: &HashMap<String, Component>,
    layouts: &HashMap<String, Component>,
    resources: &HashMap<String, String>,
) -> Diagnostics {
    let mut diagnostics = Diagnostics::default();

    let dst = dst.as_ref();
    let context = BindingContext {
        manifest,
        articles,
        resources,
    };

    for (id, article) in articles {
        if let Some(layout) = layouts.get(&article.layout) {
            let (content, diag) = process_article(id, &context, components, layout);
            diagnostics.merge(diag);

            if let Some(content) = content {
                let path = dst.join(destination::make_article_path(manifest, article, false));

                if let Some(parent) = path.parent() {
                    if let Err(err) = fs::create_dir_all(parent) {
                        diagnostics.errors.push(WriterError::Io(err).into());
                        continue;
                    }
                }

                if let Err(err) = fs::write(path, content) {
                    diagnostics.errors.push(WriterError::Io(err).into());
                    continue;
                }
            }
        } else {
            diagnostics
                .errors
                .push(WriterError::LayoutNotFound(article.layout.clone()).into());
        }
    }

    diagnostics
}
