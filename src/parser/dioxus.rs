//! Dioxus RSX (`div { class: "x", onclick: move |_| {}, "text" }`).
//!
//! This is a lenient parser for the subset of RSX that matters for
//! accessibility linting: elements, components, attributes, text, `for`
//! loops, `if`/`else` chains and `{expr}` children. Rust expressions are
//! parsed with `syn` so closures, `if` expressions and the like in attribute
//! values are handled correctly.

use proc_macro2::TokenStream;
use syn::ext::IdentExt;
use syn::parse::{ParseStream, Parser};
use syn::spanned::Spanned;
use syn::{Expr, Ident, LitStr, Token, braced, token};

use super::markup::{MElement, MNode, position};
use super::{AttrValue, HtmlAttribute};
use crate::dom::AttributeName;

/// Parse a Dioxus `rsx!` body.
pub(crate) fn parse(tokens: TokenStream) -> (Vec<MNode>, Vec<syn::Error>) {
    let parser = |input: ParseStream<'_>| parse_body(input, false).map(|(_, nodes)| nodes);
    match parser.parse2(tokens) {
        Ok(nodes) => (nodes, Vec::new()),
        Err(e) => (Vec::new(), vec![e]),
    }
}

/// Parse a sequence of attributes (only when `in_element`) and child nodes,
/// optionally separated by commas, until the stream is exhausted.
fn parse_body(
    input: ParseStream<'_>,
    in_element: bool,
) -> syn::Result<(Vec<HtmlAttribute>, Vec<MNode>)> {
    let mut attributes = Vec::new();
    let mut nodes = Vec::new();

    while !input.is_empty() {
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            continue;
        }

        // "data-custom": value
        if in_element && input.peek(LitStr) && input.peek2(Token![:]) {
            let key: LitStr = input.parse()?;
            input.parse::<Token![:]>()?;
            let value: Expr = input.parse()?;
            attributes.push(attribute(&key.value(), key.span(), &value));
            continue;
        }

        // "text", possibly with `{interpolation}`
        if input.peek(LitStr) {
            let text: LitStr = input.parse()?;
            nodes.push(text_node(&text.value()));
            continue;
        }

        // ..spread_attributes
        if input.peek(Token![..]) {
            input.parse::<Token![..]>()?;
            input.parse::<Expr>()?;
            continue;
        }

        if input.peek(Token![for]) {
            nodes.push(parse_for(input)?);
            continue;
        }

        if input.peek(Token![if]) {
            nodes.push(parse_if(input)?);
            continue;
        }

        // {expression}
        if input.peek(token::Brace) {
            let content;
            braced!(content in input);
            content.parse::<TokenStream>()?;
            nodes.push(MNode::Dynamic);
            continue;
        }

        if input.peek(Ident::peek_any) {
            // name: value
            if in_element && is_attribute_colon(input) {
                let key = Ident::parse_any(input)?;
                input.parse::<Token![:]>()?;
                let value: Expr = input.parse()?;
                attributes.push(attribute(&key.to_string(), key.span(), &value));
                continue;
            }
            // Shorthand attribute: `div { class, id }` (a variable of the same name).
            if in_element && input.peek(Ident) && (input.peek2(Token![,]) || is_last_token(input)) {
                let key: Ident = input.parse()?;
                let (line, column) = position(key.span().start());
                attributes.push(HtmlAttribute {
                    name: AttributeName::parse(&key.to_string()),
                    value: AttrValue::Dynamic,
                    line,
                    column,
                });
                continue;
            }
            nodes.push(parse_element(input)?);
            continue;
        }

        return Err(input.error("unexpected token in rsx!"));
    }

    Ok((attributes, nodes))
}

/// `ident:` but not `ident::` (a path).
fn is_attribute_colon(input: ParseStream<'_>) -> bool {
    let fork = input.fork();
    fork.call(Ident::parse_any).is_ok() && fork.peek(Token![:]) && !fork.peek(Token![::])
}

fn is_last_token(input: ParseStream<'_>) -> bool {
    let fork = input.fork();
    fork.call(Ident::parse_any).is_ok() && fork.is_empty()
}

/// `div { … }`, `MyComponent { … }`, `crate::ui::Card::<T> { … }`
fn parse_element(input: ParseStream<'_>) -> syn::Result<MNode> {
    // Also accepts generics on a component: `Comp::<T> {}`.
    let path: syn::Path = input.parse()?;
    let (line, column) = position(path.span().start());
    let name = match path.segments.first() {
        Some(segment)
            if path.segments.len() == 1
                && path.leading_colon.is_none()
                && segment.arguments.is_none() =>
        {
            segment.ident.unraw().to_string()
        }
        // A path or generic name is always a component, never an HTML element.
        _ => quote_path(&path),
    };
    if !input.peek(token::Brace) {
        return Err(input.error("expected `{` after element or component name"));
    }
    let content;
    braced!(content in input);
    let (attributes, children) = parse_body(&content, true)?;
    Ok(MNode::Element(MElement {
        name,
        line,
        column,
        attributes,
        self_closing: children.is_empty(),
        children,
    }))
}

fn quote_path(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// `for pat in expr { body }`
fn parse_for(input: ParseStream<'_>) -> syn::Result<MNode> {
    input.parse::<Token![for]>()?;
    syn::Pat::parse_multi_with_leading_vert(input)?;
    input.parse::<Token![in]>()?;
    input.call(Expr::parse_without_eager_brace)?;
    let content;
    braced!(content in input);
    let (_, children) = parse_body(&content, false)?;
    Ok(MNode::Fragment(children))
}

/// `if cond { body } else if cond { body } else { body }`
fn parse_if(input: ParseStream<'_>) -> syn::Result<MNode> {
    let mut branches = Vec::new();
    loop {
        input.parse::<Token![if]>()?;
        input.call(Expr::parse_without_eager_brace)?;
        let content;
        braced!(content in input);
        branches.extend(parse_body(&content, false)?.1);

        if !input.peek(Token![else]) {
            break;
        }
        input.parse::<Token![else]>()?;
        if input.peek(Token![if]) {
            continue;
        }
        let content;
        braced!(content in input);
        branches.extend(parse_body(&content, false)?.1);
        break;
    }
    Ok(MNode::Fragment(branches))
}

/// Dioxus format strings interpolate with `{expr}`; `{{` is an escaped brace.
fn is_interpolated(text: &str) -> bool {
    text.replace("{{", "").replace("}}", "").contains('{')
}

fn text_node(text: &str) -> MNode {
    if is_interpolated(text) {
        MNode::Dynamic
    } else {
        MNode::Text(text.to_string())
    }
}

fn attribute(key: &str, span: proc_macro2::Span, value: &Expr) -> HtmlAttribute {
    let (line, column) = position(span.start());
    let value = match value {
        Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(s) if is_interpolated(&s.value()) => AttrValue::Dynamic,
            syn::Lit::Str(s) => AttrValue::Static(s.value()),
            syn::Lit::Int(i) => AttrValue::Static(i.base10_digits().to_string()),
            syn::Lit::Float(f) => AttrValue::Static(f.base10_digits().to_string()),
            syn::Lit::Bool(b) => AttrValue::Static(b.value.to_string()),
            _ => AttrValue::Dynamic,
        },
        _ => AttrValue::Dynamic,
    };
    HtmlAttribute {
        name: AttributeName::parse(key),
        value,
        line,
        column,
    }
}
