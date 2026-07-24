pub mod connection;
pub mod context;
pub mod html;

#[derive(Debug, PartialEq, Eq)]
pub enum DirectiveContext {
    Page,
    Article,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DirectiveKind {
    Binding,
    Destination,
    StaticResource,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TargetKind {
    Attribute,
    Property,
}

#[derive(Debug)]
pub struct Directive {
    pub kind: DirectiveKind,
    pub source: String,
    pub target: String,
    pub target_kind: TargetKind,
}
