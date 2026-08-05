use dom_query::{Document, Selection};

pub fn clone_template(element: &Selection<'_>) -> Document {
    // dom_query doesn't expose the template's contents through inner_html.
    // This is a workaround - we rename the element to something else, remove
    // it from the tree, and parse it into a fragment, which does allow us to
    // use the template as intended
    element.rename("a-a");

    Document::fragment(element.html())
}
