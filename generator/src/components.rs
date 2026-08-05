pub mod article;
pub mod article_list;
pub mod article_preview;
pub mod discovery;
pub mod template;

use std::collections::HashMap;

use crate::directives::Directive;

#[derive(Debug)]
pub struct Component {
    pub content: String,
    pub directives: HashMap<u8, Vec<Directive>>,
}
