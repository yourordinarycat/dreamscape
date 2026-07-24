use dom_query::Selection;

use super::{
    Directive, DirectiveContext, DirectiveKind, TargetKind,
    context::{
        BLG_ARTICLE_LIST_TAG, BLG_ARTICLE_TAG, BLG_NEXT_ARTICLE_TAG, BLG_PREVIOUS_ARTICLE_TAG,
    },
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

pub fn get_directive_context(node: &Selection) -> DirectiveContext {
    let relevant_ancestor = node.ancestors(None).iter().any(|ancestor| {
        ancestor.is(BLG_ARTICLE_TAG)
            || ancestor.is(BLG_ARTICLE_LIST_TAG)
            || ancestor.is(BLG_PREVIOUS_ARTICLE_TAG)
            || ancestor.is(BLG_NEXT_ARTICLE_TAG)
    });

    if relevant_ancestor {
        DirectiveContext::Article
    } else {
        DirectiveContext::Page
    }
}
