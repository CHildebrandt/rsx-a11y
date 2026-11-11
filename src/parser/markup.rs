//! A small syntax-independent markup tree that both front ends (JSX via
//! rstml, and Dioxus RSX) produce, and its lowering into [`HtmlElement`]s.

use proc_macro2::LineColumn;

use super::{HtmlAttribute, HtmlElement};
use crate::dom::Tag;

/// A node in a parsed markup tree.
#[derive(Debug, Clone)]
pub(crate) enum MNode {
    /// An element or component (`<div>`, `<MyComponent>`, `div {}`).
    Element(MElement),
    /// Static text.
    Text(String),
    /// Content only known at runtime (a Rust expression, interpolated text…).
    Dynamic,
    /// A transparent group of nodes (`<>…</>`, an `if`/`for` body…).
    Fragment(Vec<MNode>),
}

/// An element or component.
#[derive(Debug, Clone)]
pub(crate) struct MElement {
    /// The name as written (`div`, `MyComponent`, `leptos_router::A`).
    pub name: String,
    /// Position of the start of the element (1-based column).
    pub line: usize,
    pub column: usize,
    pub attributes: Vec<HtmlAttribute>,
    pub children: Vec<MNode>,
    pub self_closing: bool,
}

/// Convert a proc-macro2 start position to (line, 1-based column).
pub(crate) fn position(lc: LineColumn) -> (usize, usize) {
    (lc.line, lc.column + 1)
}

/// Flatten `nodes` into `out`, producing one [`HtmlElement`] per known HTML
/// element. Components are not elements themselves, but their children are
/// still linted.
pub(crate) fn lower(nodes: &[MNode], file: &str, out: &mut Vec<HtmlElement>) {
    for node in nodes {
        match node {
            MNode::Element(el) => {
                if let Ok(tag) = el.name.parse::<Tag>() {
                    out.push(HtmlElement {
                        tag,
                        attributes: el.attributes.clone(),
                        is_self_closing: el.self_closing,
                        has_children: el.children.iter().any(has_content),
                        line: el.line,
                        column: el.column,
                        file: file.to_string(),
                    });
                }
                lower(&el.children, file, out);
            }
            MNode::Fragment(children) => lower(children, file, out),
            MNode::Text(_) | MNode::Dynamic => {}
        }
    }
}

/// Whether a child node can contribute content (whitespace-only text cannot).
fn has_content(node: &MNode) -> bool {
    match node {
        MNode::Text(text) => !text.trim().is_empty(),
        MNode::Fragment(children) => children.iter().any(has_content),
        MNode::Element(_) | MNode::Dynamic => true,
    }
}
