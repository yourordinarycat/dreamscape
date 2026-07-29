pub mod discovery;

use std::collections::HashMap;

use crate::directives::Directive;

#[derive(Debug)]
pub struct Component {
    pub content: String,
    pub directives: HashMap<u8, Vec<Directive>>,
}
