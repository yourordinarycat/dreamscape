use dom_query::{Document, Selection};

use crate::{
    components::template::clone_template,
    diagnostics::Diagnostics,
    directives::{
        self, DirectiveContext, binding::BindingContext, connection::CONNECTION_ID_ATTR_SELECTOR,
    },
};

pub struct ArticleListElement<'dom> {
    element: &'dom Selection<'dom>,
}

impl<'dom> ArticleListElement<'dom> {
    pub fn new(element: &'dom Selection<'_>) -> Self {
        Self { element }
    }

    pub fn inflate(&mut self, context: &BindingContext) -> Diagnostics {
        let mut diagnostics = Diagnostics::default();
        let item_template = self.element.select_single("template");

        let template_data = if item_template.exists() {
            let fragment = clone_template(&item_template);
            let all_children = fragment.select("*");
            let directive_map = directives::html::extract_all(all_children.iter());

            item_template.remove();

            Some((fragment.html(), directive_map))
        } else {
            None
        };

        for (id, article) in context.articles {
            if article.default {
                continue;
            }

            self.element.append_html("<li></li>");
            let li = self.element.children().last();

            if let Some((template, directive_map)) = &template_data {
                let fragment = Document::fragment(template.to_string());

                diagnostics.errors.extend(
                    directives::html::apply_all(
                        fragment.select(CONNECTION_ID_ATTR_SELECTOR).iter(),
                        id,
                        &directive_map,
                        context,
                        DirectiveContext::Article,
                    )
                    .into_iter()
                    .map(|e| e.into()),
                );

                let root = fragment.html_root().first_child().unwrap();
                for node in root.children() {
                    li.append_html(node.html());
                }
            } else {
                li.set_text(&article.title);
            }
        }

        self.element.rename("ul");
        self.element.add_class("article-list");

        diagnostics
    }
}
