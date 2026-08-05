use dom_query::Selection;

use crate::{
    components::template::clone_template,
    diagnostics::Diagnostics,
    directives::{
        self, DirectiveContext, binding::BindingContext, connection::CONNECTION_ID_ATTR_SELECTOR,
    },
};

pub struct ArticlePreviewElement<'dom> {
    element: &'dom Selection<'dom>,
}

impl<'dom> ArticlePreviewElement<'dom> {
    pub fn new(element: &'dom Selection<'_>) -> Self {
        Self { element }
    }

    pub fn inflate(&mut self, id: &str, context: &BindingContext) -> Diagnostics {
        let mut diagnostics = Diagnostics::default();
        let template = self.element.select_single("template");

        if template.exists() {
            let fragment = clone_template(&template);
            template.remove();

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
        }

        diagnostics
    }
}
