use dom_query::{Document, Selection};
use indexmap::IndexMap;
use std::{collections::HashMap, fs, path::Path};

use crate::{
    articles::Article,
    directives::{
        binding::BindingContext,
        connection::{CONNECTION_ID_ATTR, CONNECTION_ID_ATTR_SELECTOR},
        context::{
            BLG_ARTICLE_LIST_TAG, BLG_ARTICLE_TAG, BLG_NEXT_ARTICLE_TAG, BLG_PREVIOUS_ARTICLE_TAG,
        },
        destination, html,
    },
    layouts::Layout,
    manifest::Manifest,
};

fn apply_directives<'a>(
    nodes: impl Iterator<Item = Selection<'a>>,
    id: &str,
    context: &BindingContext,
    layout: &Layout,
) {
    for node in nodes {
        let cid: u8 = node.attr(CONNECTION_ID_ATTR).unwrap().parse().unwrap();
        node.remove_attr(CONNECTION_ID_ATTR);

        let (directive_context, directives) = layout.directives.get(&cid).unwrap();

        for directive in directives {
            let value = context.get(id, directive, *directive_context);
            if let Some(value) = value {
                html::apply(&node, directive, &value);
            }
        }
    }
}

fn process_article(
    id: &str,
    context: &BindingContext,
    layout: &Layout,
) -> Result<String, Box<dyn std::error::Error>> {
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
            apply_directives(
                node.select(CONNECTION_ID_ATTR_SELECTOR).iter(),
                &previous.id,
                context,
                layout,
            );

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
            apply_directives(
                node.select(CONNECTION_ID_ATTR_SELECTOR).iter(),
                &next.id,
                context,
                layout,
            );

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

            apply_directives(
                li.select(CONNECTION_ID_ATTR_SELECTOR).iter(),
                &article.id,
                context,
                layout,
            );
        }
    }

    // Process remaining elements with connection IDs
    let nodes = document.select(CONNECTION_ID_ATTR_SELECTOR).iter();
    apply_directives(nodes, id, context, layout);

    Ok(document.html().to_string())
}

pub fn write_to(
    dst: impl AsRef<Path>,
    manifest: &Manifest,
    articles: &IndexMap<String, Article>,
    layouts: &HashMap<String, Layout>,
    resources: &HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let dst = dst.as_ref();
    let context = BindingContext {
        manifest,
        articles,
        resources,
    };

    for (id, article) in articles {
        if let Some(layout) = layouts.get(&article.layout) {
            let content = process_article(id, &context, layout)?;
            let path = dst.join(destination::make_article_path(manifest, article, false));

            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(path, content)?;
        } else {
            println!("Unable to find layout {}", article.layout);
        }
    }

    Ok(())
}
