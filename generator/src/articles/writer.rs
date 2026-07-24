use dom_query::{Document, Selection};
use icu::{
    calendar::{Date, Iso},
    datetime::{DateTimeFormatter, fieldsets::YMD},
    locale::Locale,
};
use indexmap::IndexMap;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    articles::Article,
    directives::{
        Directive, DirectiveContext, DirectiveKind, TargetKind,
        connection::{CONNECTION_ID_ATTR, CONNECTION_ID_ATTR_SELECTOR},
        context::{
            BLG_ARTICLE_LIST_TAG, BLG_ARTICLE_TAG, BLG_NEXT_ARTICLE_TAG, BLG_PREVIOUS_ARTICLE_TAG,
        },
    },
    layouts::Layout,
    manifest::Manifest,
    resources::href,
};

fn apply_directives<'a>(
    nodes: impl Iterator<Item = Selection<'a>>,
    manifest: &Manifest,
    article: &Article,
    layout: &Layout,
    articles: &IndexMap<String, Article>,
    resources: &HashMap<String, String>,
) {
    for node in nodes {
        let cid: u8 = node.attr(CONNECTION_ID_ATTR).unwrap().parse().unwrap();
        node.remove_attr(CONNECTION_ID_ATTR);

        let (context, directives) = layout.directives.get(&cid).unwrap();

        for directive in directives {
            match directive.kind {
                DirectiveKind::Binding => {
                    if *context == DirectiveContext::Page {
                        process_page_binding(&node, directive, manifest, article)
                    } else {
                        process_article_binding(&node, directive, manifest, article)
                    }
                }
                DirectiveKind::Destination => {
                    process_destination(&node, directive, articles, manifest)
                }
                DirectiveKind::StaticResource => process_resource(&node, directive, resources),
            };
        }
    }
}

fn process_article(
    manifest: &Manifest,
    article: &Article,
    layout: &Layout,
    articles: &IndexMap<String, Article>,
    resources: &HashMap<String, String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let document = Document::from(&*layout.content);

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

    let prev_idx = articles.get_index_of(&article.id).unwrap() + 1;
    let prev_article_nodes = document.select(BLG_PREVIOUS_ARTICLE_TAG);

    if let Some((_, previous)) = articles.get_index(prev_idx)
        && !previous.default
    {
        for node in prev_article_nodes.iter() {
            apply_directives(
                node.select(CONNECTION_ID_ATTR_SELECTOR).iter(),
                manifest,
                previous,
                layout,
                articles,
                resources,
            );

            node.replace_with_selection(&node.children());
        }
    } else {
        prev_article_nodes.remove();
    }

    let next_idx_opt = articles.get_index_of(&article.id).unwrap().checked_sub(1);
    let next_article_nodes = document.select(BLG_NEXT_ARTICLE_TAG);

    if let Some(next_idx) = next_idx_opt
        && let Some((_, next)) = articles.get_index(next_idx)
        && !next.default
    {
        for node in next_article_nodes.iter() {
            apply_directives(
                node.select(CONNECTION_ID_ATTR_SELECTOR).iter(),
                manifest,
                next,
                layout,
                articles,
                resources,
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

        for (_, article) in articles {
            if article.default {
                continue;
            }

            node.append_html("<li></li>");
            let li = node.children().last();

            li.append_html(template.clone());

            apply_directives(
                li.select(CONNECTION_ID_ATTR_SELECTOR).iter(),
                manifest,
                article,
                layout,
                articles,
                resources,
            );
        }
    }

    // Process remaining elements with connection IDs
    let nodes = document.select(CONNECTION_ID_ATTR_SELECTOR).iter();
    apply_directives(nodes, manifest, article, layout, articles, resources);

    Ok(document.html().to_string())
}

fn apply_directive(node: &Selection, directive: &Directive, value: &str) {
    if directive.target_kind == TargetKind::Attribute {
        node.set_attr(&*directive.target, value);
    } else if directive.target == "content" {
        node.set_html(value);
    }
}

fn format_date(date: &Date<Iso>) -> String {
    let year = date.era_year().year;
    let month = date.month().ordinal;
    let day = date.day_of_month().0;

    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn format_date_display(date: &Date<Iso>, locale_str: &str) -> String {
    let locale = Locale::try_from_str(locale_str).expect("Invalid locale string.");

    let dtf = DateTimeFormatter::try_new(locale.into(), YMD::long())
        .expect("Failed to initialize formatter");

    dtf.format(date).to_string()
}

fn process_common_binding(
    directive: &Directive,
    manifest: &Manifest,
    article: &Article,
) -> Option<String> {
    match directive.source.as_str() {
        "author" => Some(article.author.clone()),
        "publishDate" => Some(format_date(&article.created)),
        "publishDisplayDate" => Some(format_date_display(
            &article.created,
            &manifest.default_language,
        )),
        "updateDate" => Some(format_date(&article.updated)),
        "updateDisplayDate" => Some(format_date_display(
            &article.updated,
            &manifest.default_language,
        )),
        "language" => Some(manifest.default_language.clone()),
        "url" => Some(href::normalize(&make_article_full_path(
            manifest, article, true,
        ))),
        _ => None,
    }
}

fn process_article_binding(
    node: &Selection,
    directive: &Directive,
    manifest: &Manifest,
    article: &Article,
) {
    let value_opt = match directive.source.as_str() {
        "title" => {
            if article.default {
                Some(manifest.title.clone())
            } else {
                Some(article.title.clone())
            }
        }
        _ => process_common_binding(directive, manifest, article),
    };

    if let Some(value) = value_opt {
        apply_directive(node, directive, &value);
    }
}

fn process_page_binding(
    node: &Selection,
    directive: &Directive,
    manifest: &Manifest,
    article: &Article,
) {
    let value_opt = match directive.source.as_str() {
        "title" => {
            if article.default {
                Some(manifest.title.clone())
            } else {
                Some(format!("{} - {}", article.short_title, manifest.title))
            }
        }
        _ => process_common_binding(directive, manifest, article),
    };

    if let Some(value) = value_opt {
        apply_directive(node, directive, &value);
    }
}

fn make_article_path(article: &Article, default_is_empty: bool) -> PathBuf {
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
        buf.push(format_date(&article.created).replace('-', "/"));
        buf.push(id);

        buf
    }
}

fn make_article_full_path(
    manifest: &Manifest,
    article: &Article,
    default_is_empty: bool,
) -> PathBuf {
    let article_path = make_article_path(&article, default_is_empty);

    if let Some(base_url) = &manifest.base_path {
        let base_path = Path::new(base_url);
        base_path.join(article_path)
    } else {
        article_path
    }
}

fn process_destination(
    node: &Selection,
    directive: &Directive,
    articles: &IndexMap<String, Article>,
    manifest: &Manifest,
) {
    let referenced_article = articles
        .get(&directive.source)
        .expect("Destination references an article that doesn't exist.");

    let full_path = make_article_full_path(manifest, &referenced_article, true);
    apply_directive(node, directive, &href::normalize(&full_path));
}

fn process_resource(node: &Selection, directive: &Directive, resources: &HashMap<String, String>) {
    let value = resources
        .get(&directive.source)
        .expect("Resource should exist.");

    apply_directive(node, directive, value);
}

pub fn write_to(
    dst: impl AsRef<Path>,
    manifest: &Manifest,
    articles: &IndexMap<String, Article>,
    layouts: &HashMap<String, Layout>,
    resources: &HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let dst = dst.as_ref();

    for (_, article) in articles {
        if let Some(layout) = layouts.get(&article.layout) {
            let content = process_article(manifest, article, layout, articles, resources)?;
            let path = dst.join(make_article_path(article, false));

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
