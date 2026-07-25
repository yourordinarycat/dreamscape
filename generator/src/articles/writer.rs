use dom_query::{Document, Selection};
use indexmap::IndexMap;
use std::{collections::HashMap, fs, path::Path};

use crate::{
    articles::Article,
    diagnostics::Diagnostics,
    directives::{
        binding::{BindingContext, DirectiveError},
        connection::{CONNECTION_ID_ATTR, CONNECTION_ID_ATTR_SELECTOR},
        context::{
            BLG_ARTICLE_LIST_TAG, BLG_ARTICLE_TAG, BLG_NEXT_ARTICLE_TAG, BLG_PREVIOUS_ARTICLE_TAG,
        },
        destination, html,
    },
    layouts::Layout,
    manifest::Manifest,
};

#[derive(thiserror::Error, Debug)]
pub enum WriterError {
    #[error("Unable to find layout with provided ID: {0}")]
    LayoutNotFound(String),

    #[error(transparent)]
    DirectiveError(#[from] DirectiveError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

fn apply_directives<'a>(
    nodes: impl Iterator<Item = Selection<'a>>,
    id: &str,
    context: &BindingContext,
    layout: &Layout,
) -> Vec<DirectiveError> {
    let mut errors: Vec<DirectiveError> = Vec::new();

    for node in nodes {
        let cid: u8 = node.attr(CONNECTION_ID_ATTR).unwrap().parse().unwrap();
        node.remove_attr(CONNECTION_ID_ATTR);

        let (directive_context, directives) = layout.directives.get(&cid).unwrap();

        for directive in directives {
            match context.get(id, directive, *directive_context) {
                Ok(value) => html::apply(&node, directive, &value),
                Err(err) => errors.push(err),
            };
        }
    }

    errors
}

fn process_article(
    id: &str,
    context: &BindingContext,
    layout: &Layout,
) -> (Option<String>, Diagnostics) {
    let mut diagnostics = Diagnostics::default();

    let mut apply_all = |nodes: Selection<'_>, id: &str| {
        let errs = apply_directives(nodes.iter(), id, context, layout);

        diagnostics.errors.extend(
            errs.into_iter()
                .map(|e| WriterError::DirectiveError(e.into()).into()),
        );
    };

    let document = Document::from(&*layout.content);
    let article = context.articles.get(id).unwrap();

    let article_nodes = document.select(BLG_ARTICLE_TAG).iter();
    for node in article_nodes {
        let html = Document::from(&*article.content).html();
        if !article.default {
            node.rename("article");
            node.append_html(html);
        } else {
            node.replace_with_html(html);
        }
    }

    let prev_idx = context.articles.get_index_of(id).unwrap() + 1;
    let prev_article_nodes = document.select(BLG_PREVIOUS_ARTICLE_TAG);

    if let Some((_, previous)) = context.articles.get_index(prev_idx)
        && !previous.default
    {
        for node in prev_article_nodes.iter() {
            apply_all(node.select(CONNECTION_ID_ATTR_SELECTOR), &previous.id);
            node.replace_with_selection(&node.children());
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
            apply_all(node.select(CONNECTION_ID_ATTR_SELECTOR), &next.id);
            node.replace_with_selection(&node.children());
        }
    } else {
        next_article_nodes.remove();
    }

    let article_list_nodes = document.select(BLG_ARTICLE_LIST_TAG).iter();
    for node in article_list_nodes {
        node.rename("ul");
        node.add_class("article-list");

        let template = node.inner_html();
        node.children().remove();

        for (_, article) in context.articles {
            if article.default {
                continue;
            }

            node.append_html("<li></li>");
            let li = node.children().last();

            li.append_html(template.clone());

            apply_all(li.select(CONNECTION_ID_ATTR_SELECTOR), &article.id);
        }
    }

    // Process remaining elements with connection IDs
    let nodes = document.select(CONNECTION_ID_ATTR_SELECTOR);
    apply_all(nodes, id);

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
    layouts: &HashMap<String, Layout>,
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
            let (content, diag) = process_article(id, &context, layout);
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
