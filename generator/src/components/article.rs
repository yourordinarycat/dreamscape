use dom_query::{Document, Selection};

use crate::{
    components::template::clone_template,
    diagnostics::Diagnostics,
    directives::{
        self, DirectiveContext, binding::BindingContext, connection::CONNECTION_ID_ATTR_SELECTOR,
    },
};

pub struct ArticleElement<'dom> {
    element: &'dom Selection<'dom>,
}

impl<'dom> ArticleElement<'dom> {
    pub fn new(element: &'dom Selection<'_>) -> Self {
        Self { element }
    }

    pub fn inflate(&mut self, id: &str, context: &BindingContext) -> Diagnostics {
        let mut diagnostics = Diagnostics::default();

        let article = context.articles.get(id).unwrap();
        let header_template = self
            .element
            .select_single("template[slot=\"header-template\"]");

        if header_template.exists() {
            let fragment = clone_template(&header_template);
            header_template.remove();

            let all_children = fragment.select("*");
            let directive_map = directives::html::extract_all(all_children.iter());

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
                self.element.append_html(node.html());
            }
        } else {
            self.element.append_html("<h1></h1>");
            let h1 = self.element.children().last();
            h1.set_text(&article.title);
        }

        let html = Document::from(&*article.content).html();
        self.element.append_html(html);

        if !article.default {
            self.element.rename("article");
        } else {
            self.element.replace_with_html(self.element.inner_html());
        }

        diagnostics
    }
}
