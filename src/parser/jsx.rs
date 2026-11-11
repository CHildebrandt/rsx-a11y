//! JSX-style markup (`<div class="x">…</div>`) via rstml.

use std::cell::RefCell;
use std::rc::Rc;

use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use rstml::node::{KeyedAttribute, KeyedAttributeValue, Node, NodeAttribute, NodeBlock};
use rstml::{Parser, ParserConfig};
use syn::parse::ParseStream;
use syn::spanned::Spanned;

use super::markup::{MElement, MNode, position};
use super::{AttrValue, HtmlAttribute};
use crate::dom::AttributeName;

/// Prefix of the identifiers that stand in for stashed non-Rust blocks.
const PLACEHOLDER: &str = "__rsx_a11y_block_";

/// Parse JSX-style markup, recovering from errors where possible.
pub(crate) fn parse(tokens: TokenStream) -> (Vec<MNode>, Vec<syn::Error>) {
    // Blocks that are not valid Rust — Yew's `{ for items }` iterator syntax,
    // or markup directly inside `if cond { <p/> }` — are swapped for a
    // placeholder identifier so they do not fail the whole macro. Their
    // original tokens are kept in `stash` and examined during conversion.
    let stash: Rc<RefCell<Vec<TokenStream>>> = Rc::default();
    let config = {
        let stash = Rc::clone(&stash);
        ParserConfig::new().transform_block(move |input: ParseStream<'_>| {
            let fork = input.fork();
            if fork.call(syn::Block::parse_within).is_ok() && fork.is_empty() {
                return Ok(None);
            }
            // Consume token by token: parsing a whole `TokenStream` would move
            // the cursor out of this scope, which rstml's `advance_to` rejects.
            let mut body = TokenStream::new();
            while !input.is_empty() {
                body.extend([input.parse::<TokenTree>()?]);
            }
            let mut stash = stash.borrow_mut();
            let placeholder =
                Ident::new(&format!("{PLACEHOLDER}{}", stash.len()), Span::call_site());
            stash.push(body);
            Ok(Some(TokenTree::Ident(placeholder).into()))
        })
    };
    let (nodes, diagnostics) = Parser::new(config).parse_recoverable(tokens).split_vec();
    let errors = diagnostics.into_iter().map(syn::Error::from).collect();
    let stash = stash.borrow();
    (Converter { stash: &stash }.nodes(&nodes), errors)
}

struct Converter<'a> {
    stash: &'a [TokenStream],
}

impl Converter<'_> {
    fn nodes(&self, nodes: &[Node]) -> Vec<MNode> {
        nodes.iter().filter_map(|n| self.node(n)).collect()
    }

    fn node(&self, node: &Node) -> Option<MNode> {
        Some(match node {
            Node::Element(el) => {
                let (line, column) = position(el.open_tag.token_lt.span().start());
                let mut has_spread = false;
                let mut attributes = Vec::new();
                for attr in el.attributes() {
                    match attr {
                        NodeAttribute::Attribute(keyed) => attributes.push(attribute(keyed)),
                        NodeAttribute::Block(block) => match shorthand_attribute(block) {
                            Some(attr) => attributes.push(attr),
                            // `{..spread}` or a computed attribute.
                            None => has_spread = true,
                        },
                    }
                }
                MNode::Element(MElement {
                    name: el.name().to_string(),
                    has_spread,
                    line,
                    column,
                    attributes,
                    children: self.nodes(&el.children),
                    self_closing: el.close_tag.is_none(),
                })
            }
            Node::Fragment(fragment) => MNode::Fragment(self.nodes(&fragment.children)),
            Node::Text(text) => MNode::Text(text.value_string()),
            Node::RawText(raw) => MNode::Text(raw.to_string_best()),
            Node::Block(block) => self.block(block),
            Node::Comment(_) | Node::Doctype(_) | Node::Custom(_) => return None,
        })
    }

    fn block(&self, block: &NodeBlock) -> MNode {
        let NodeBlock::ValidBlock(b) = block else {
            return MNode::Dynamic;
        };
        match b.stmts.as_slice() {
            // `{"text"}` — Yew's way of writing static text.
            [syn::Stmt::Expr(syn::Expr::Lit(lit), None)] => match &lit.lit {
                syn::Lit::Str(s) => MNode::Text(s.value()),
                _ => MNode::Dynamic,
            },
            [syn::Stmt::Expr(syn::Expr::Path(path), None)] => path
                .path
                .get_ident()
                .and_then(|ident| {
                    let index = ident.to_string().strip_prefix(PLACEHOLDER)?.parse().ok()?;
                    self.stash.get::<usize>(index)
                })
                .map_or(MNode::Dynamic, stashed),
            _ => MNode::Dynamic,
        }
    }
}

/// A block that was not valid Rust: markup if it starts with `<`, else
/// something dynamic (such as Yew's `{ for items }`).
fn stashed(body: &TokenStream) -> MNode {
    let starts_with_markup = matches!(
        body.clone().into_iter().next(),
        Some(TokenTree::Punct(p)) if p.as_char() == '<'
    );
    if starts_with_markup {
        let (nodes, errors) = parse(body.clone());
        if errors.is_empty() {
            return MNode::Fragment(nodes);
        }
    }
    MNode::Dynamic
}

/// Yew's shorthand `{onclick}`, meaning `onclick={onclick}`.
fn shorthand_attribute(block: &NodeBlock) -> Option<HtmlAttribute> {
    let NodeBlock::ValidBlock(b) = block else {
        return None;
    };
    let [syn::Stmt::Expr(syn::Expr::Path(path), None)] = b.stmts.as_slice() else {
        return None;
    };
    let ident = path.path.get_ident()?;
    let (line, column) = position(ident.span().start());
    Some(HtmlAttribute {
        name: AttributeName::parse(&ident.to_string()),
        value: AttrValue::Dynamic,
        line,
        column,
    })
}

fn attribute(keyed: &KeyedAttribute) -> HtmlAttribute {
    let (line, column) = position(keyed.key.span().start());
    let value = match &keyed.possible_value {
        KeyedAttributeValue::None => AttrValue::Bare,
        KeyedAttributeValue::Binding(_) => AttrValue::Dynamic,
        KeyedAttributeValue::Value(_) => keyed
            .value_literal_string()
            .map_or(AttrValue::Dynamic, AttrValue::Static),
    };
    HtmlAttribute {
        name: AttributeName::parse(&keyed.key.to_string()),
        value,
        line,
        column,
    }
}
