use std::collections::HashMap;

use dom_query::Selection;

use super::{
    Directive, DirectiveContext, DirectiveKind, TargetKind,
    binding::{BindingContext, DirectiveError},
    connection::CONNECTION_ID_ATTR,
};

fn remove_first_and_last(s: &str) -> &str {
    let mut chars = s.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}

fn parse_directive(key: &str, value: &str) -> Option<Directive> {
    if !value.starts_with("{") || !value.ends_with("}") {
        return None;
    }

    let content: Vec<&str> = remove_first_and_last(value).split(" ").collect();
    if content.len() > 2 {
        return None;
    }

    let directive_kind: Option<DirectiveKind> = match content[0] {
        "Binding" => Some(DirectiveKind::Binding),
        "Destination" => Some(DirectiveKind::Destination),
        "StaticResource" => Some(DirectiveKind::StaticResource),
        _ => None,
    };

    if directive_kind.is_none() {
        return None;
    }

    let target_kind: TargetKind = if key.starts_with("attr.") {
        TargetKind::Attribute
    } else {
        TargetKind::Property
    };

    let source = if content.len() == 1 { "" } else { content[1] };
    let target = if target_kind == TargetKind::Attribute {
        key.strip_prefix("attr.").unwrap()
    } else {
        key
    };

    Some(Directive {
        kind: directive_kind.unwrap(),
        source: source.to_string(),
        target: target.to_string(),
        target_kind,
    })
}

/// An iterable that returns each of the binding directives found in the node's
/// attributes. The attributes that originally contained directives are removed
/// from the node.
pub fn from_node(node: &Selection) -> impl Iterator<Item = Directive> {
    node.attrs().into_iter().filter_map(|attr| {
        let name = &attr.name.local;
        let value = &attr.value;

        let parsed = parse_directive(name, value);
        if parsed.is_some() {
            node.remove_attr(name);
        }

        parsed
    })
}

pub fn extract_all<'a>(nodes: impl Iterator<Item = Selection<'a>>) -> HashMap<u8, Vec<Directive>> {
    let mut directive_map: HashMap<u8, Vec<Directive>> = HashMap::new();
    let mut curr_cid: u8 = 0;

    for node in nodes {
        let directives: Vec<_> = from_node(&node).collect();

        if directives.len() > 0 {
            directive_map.insert(curr_cid, directives);

            node.set_attr(CONNECTION_ID_ATTR, &curr_cid.to_string());
            curr_cid += 1;
        }
    }

    directive_map
}

/// Applies a directive using the provided value to the node.
pub fn apply(node: &Selection, directive: &Directive, value: &str) {
    if directive.target_kind == TargetKind::Attribute {
        node.set_attr(&*directive.target, value);
    } else if directive.target == "content" {
        node.set_html(value);
    }
}

pub fn apply_all<'a>(
    nodes: impl Iterator<Item = Selection<'a>>,
    id: &str,
    directives: &HashMap<u8, Vec<Directive>>,
    context: &BindingContext,
    directive_context: DirectiveContext,
) -> Vec<DirectiveError> {
    let mut errors: Vec<DirectiveError> = Vec::new();

    for node in nodes {
        let cid: u8 = node.attr(CONNECTION_ID_ATTR).unwrap().parse().unwrap();
        node.remove_attr(CONNECTION_ID_ATTR);

        let directives = directives.get(&cid).unwrap();

        for directive in directives {
            match context.get(id, directive, directive_context) {
                Ok(value) => apply(&node, directive, &value),
                Err(err) => errors.push(err),
            };
        }
    }

    errors
}
