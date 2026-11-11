//! A small syntax-independent markup tree that both front ends (JSX via
//! rstml, and Dioxus RSX) produce, and its lowering into [`HtmlElement`]s.

use proc_macro2::LineColumn;

use super::{AttrValue, HtmlAttribute, HtmlElement};
use crate::dom::{AttributeName, Tag};

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
    /// Whether the element has spread attributes.
    pub has_spread: bool,
    pub children: Vec<MNode>,
    pub self_closing: bool,
}

/// The value of a literal attribute expression as a string: string, char,
/// integer, float and bool literals, plus negated numbers such as `-1`
/// (which syn parses as a unary expression, not a literal).
pub(crate) fn literal_value(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(s) => Some(s.value()),
            syn::Lit::Char(c) => Some(c.value().to_string()),
            syn::Lit::Int(i) => Some(i.base10_digits().to_string()),
            syn::Lit::Float(f) => Some(f.base10_digits().to_string()),
            syn::Lit::Bool(b) => Some(b.value.to_string()),
            _ => None,
        },
        syn::Expr::Unary(syn::ExprUnary {
            op: syn::UnOp::Neg(_),
            expr,
            ..
        }) => match expr.as_ref() {
            syn::Expr::Lit(lit) => match &lit.lit {
                syn::Lit::Int(i) => Some(format!("-{}", i.base10_digits())),
                syn::Lit::Float(f) => Some(format!("-{}", f.base10_digits())),
                _ => None,
            },
            _ => None,
        },
        syn::Expr::Paren(p) => literal_value(&p.expr),
        syn::Expr::Group(g) => literal_value(&g.expr),
        _ => None,
    }
}

/// Convert a proc-macro2 start position to (line, 1-based column).
pub(crate) fn position(lc: LineColumn) -> (usize, usize) {
    (lc.line, lc.column + 1)
}

/// Flatten `nodes` into `out`, producing one [`HtmlElement`] per known HTML
/// element. Components are not elements themselves, but their children are
/// still linted.
pub(crate) fn lower(nodes: &[MNode], file: &str, out: &mut Vec<HtmlElement>) {
    lower_into(nodes, None, file, out);
}

fn lower_into(nodes: &[MNode], parent: Option<usize>, file: &str, out: &mut Vec<HtmlElement>) {
    for node in nodes {
        match node {
            MNode::Element(el) => match el.name.parse::<Tag>() {
                Ok(tag) => {
                    let index = out.len();
                    let mut content = Content::default();
                    content.collect(&el.children);
                    out.push(HtmlElement {
                        tag,
                        attributes: el.attributes.clone(),
                        has_spread_attributes: el.has_spread,
                        is_self_closing: el.self_closing,
                        has_children: el.children.iter().any(has_content),
                        text: content
                            .text
                            .split_whitespace()
                            .collect::<Vec<_>>()
                            .join(" "),
                        has_dynamic_content: content.dynamic,
                        index,
                        parent,
                        line: el.line,
                        column: el.column,
                        file: file.to_string(),
                    });
                    lower_into(&el.children, Some(index), file, out);
                }
                // A component: lint its children, which it may render
                // anywhere, so they get no parent.
                Err(_) => lower_into(&el.children, None, file, out),
            },
            MNode::Fragment(children) => lower_into(children, parent, file, out),
            MNode::Text(_) | MNode::Dynamic => {}
        }
    }
}

/// Accumulates the static text of a subtree.
#[derive(Default)]
struct Content {
    text: String,
    dynamic: bool,
}

impl Content {
    fn collect(&mut self, nodes: &[MNode]) {
        for node in nodes {
            match node {
                MNode::Text(text) => {
                    self.text.push(' ');
                    self.text.push_str(text);
                }
                MNode::Dynamic => self.dynamic = true,
                MNode::Fragment(children) => self.collect(children),
                MNode::Element(el) => match el.name.parse::<Tag>() {
                    Ok(Tag::Img) => {
                        let alt = el.attributes.iter().find(|a| a.name == AttributeName::Alt);
                        match alt.map(|a| &a.value) {
                            Some(AttrValue::Static(alt)) => {
                                self.text.push(' ');
                                self.text.push_str(alt);
                            }
                            Some(_) => self.dynamic = true,
                            None => {}
                        }
                    }
                    Ok(_) => {
                        self.dynamic |= el.has_spread;
                        self.collect(&el.children);
                    }
                    // A component can render anything.
                    Err(_) => self.dynamic = true,
                },
            }
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
