//! Parser for Rust source files and the markup inside UI macros.
//!
//! Finds `html!`/`html_nested!` (Yew), `view!` (Leptos) and `rsx!` (Dioxus)
//! invocations — including ones nested inside other macros, blocks, closures
//! and attribute values — and extracts the HTML elements they contain.
//!
//! Two markup syntaxes are understood:
//!
//! * JSX-style markup (`<div class="x">…</div>`), used by Yew, Leptos and
//!   JSX-flavoured `rsx!` macros, parsed with [`rstml`].
//! * Dioxus RSX (`div { class: "x", "text" }`).
//!
//! An `html!`/`view!` macro that does not start with `<` (for example maud's
//! `html!` or Sycamore's `view!`) is not a syntax this crate understands and is
//! skipped rather than reported as an error.

mod dioxus;
mod jsx;
mod markup;
#[cfg(test)]
mod tests;

use std::path::Path;

use proc_macro2::{Delimiter, TokenStream, TokenTree};
use syn::visit::Visit;

use crate::dom::{AttributeName, Role, Tag};

/// Macro names whose bodies contain markup.
const MARKUP_MACROS: &[&str] = &["html", "html_nested", "view", "rsx"];

/// Represents an HTML element found in a macro invocation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct HtmlElement {
    /// Element tag name (e.g., "div", "img", "button").
    pub tag: Tag,
    /// Attributes on the element.
    pub attributes: Vec<HtmlAttribute>,
    /// Whether the element is self-closing (e.g., `<img />`).
    pub is_self_closing: bool,
    /// Whether the element has child content (text or nested elements).
    pub has_children: bool,
    /// Line number in the source file (1-based).
    pub line: usize,
    /// Column number in the source file (1-based).
    pub column: usize,
    /// The source file path.
    pub file: String,
}

impl HtmlElement {
    /// The first attribute named `name`, if any.
    pub fn attr(&self, name: &AttributeName) -> Option<&HtmlAttribute> {
        self.attributes.iter().find(|a| &a.name == name)
    }

    /// Whether an attribute named `name` is present (with any value).
    pub fn has_attr(&self, name: &AttributeName) -> bool {
        self.attr(name).is_some()
    }

    /// Whether any of the given attributes is present.
    pub fn has_any_attr(&self, names: &[AttributeName]) -> bool {
        names.iter().any(|n| self.has_attr(n))
    }

    /// The static string value of attribute `name`, if present and static.
    pub fn static_attr(&self, name: &AttributeName) -> Option<&str> {
        self.attr(name).and_then(|a| a.value.as_static())
    }

    /// The role set with an explicit, static `role` attribute (if it parses).
    pub fn explicit_role(&self) -> Option<Role> {
        self.static_attr(&AttributeName::Role)
            .and_then(|v| v.trim().parse().ok())
    }

    /// The effective role: explicit `role` attribute takes precedence,
    /// falling back to the tag's implicit role.
    pub fn role(&self) -> Option<Role> {
        self.explicit_role().or_else(|| self.tag.implicit_role())
    }

    /// Whether the element is focusable (natively interactive or has tabindex >= 0).
    pub fn is_focusable(&self) -> bool {
        self.tag.is_interactive()
            || self
                .attr(&AttributeName::TabIndex)
                .is_some_and(|a| match &a.value {
                    AttrValue::Static(v) => v.trim().parse::<i32>().is_ok_and(|i| i >= 0),
                    // Dynamic value: assume it may be focusable.
                    AttrValue::Dynamic | AttrValue::Bare => true,
                })
    }

    /// Whether there is a mouse or keyboard event handler on this element.
    pub fn has_event_handler(&self) -> bool {
        self.attributes
            .iter()
            .any(|a| a.name.is_interaction_handler())
    }
}

/// Represents an attribute on an HTML element.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct HtmlAttribute {
    /// Attribute name (e.g., "aria-label", "class", "role").
    pub name: AttributeName,
    /// Attribute value.
    pub value: AttrValue,
    /// Line number in the source file (1-based).
    pub line: usize,
    /// Column number in the source file (1-based).
    pub column: usize,
}

/// Represents the value of an HTML attribute.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[non_exhaustive]
pub enum AttrValue {
    /// A static string literal value (contents without quotes).
    Static(String),
    /// A dynamic expression (e.g., `{some_variable}`) — cannot be checked statically.
    Dynamic,
    /// No value at all, as in a boolean attribute: `<input autofocus />`.
    Bare,
}

impl AttrValue {
    /// Get the static string value, if available.
    pub fn as_static(&self) -> Option<&str> {
        match self {
            AttrValue::Static(s) => Some(s),
            AttrValue::Dynamic | AttrValue::Bare => None,
        }
    }

    /// Whether the value is the static string `expected`.
    pub fn is(&self, expected: &str) -> bool {
        self.as_static() == Some(expected)
    }
}

/// The elements found in one source file, plus any macros that could not be
/// parsed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedSource {
    /// Every HTML element found, in source order.
    pub elements: Vec<HtmlElement>,
    /// Markup macros that could not be parsed. Elements in these macros are
    /// missing from `elements`, so the file was only partially linted.
    pub errors: Vec<ParseError>,
}

/// Normalize a path to use forward slashes consistently.
fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Read and parse a Rust source file.
///
/// # Errors
///
/// Returns an error if the file cannot be read or is not valid Rust. Markup
/// errors inside individual macros are reported in [`ParsedSource::errors`].
pub fn parse_file(path: &Path) -> Result<ParsedSource, ParseError> {
    let file_path = normalize_path(path);
    let source = std::fs::read_to_string(path)
        .map_err(|e| ParseError::new(ParseErrorKind::Io, &file_path, e.to_string()))?;
    parse_source(&source, &file_path)
}

/// Parse Rust source code and extract HTML elements from supported macros.
///
/// # Errors
///
/// Returns an error if `source` is not valid Rust. Markup errors inside
/// individual macros are reported in [`ParsedSource::errors`].
pub fn parse_source(source: &str, file_path: &str) -> Result<ParsedSource, ParseError> {
    let syntax_tree = syn::parse_file(source).map_err(|e| {
        let line = e.span().start().line;
        ParseError::new(ParseErrorKind::Rust, file_path, e.to_string()).at_line(line)
    })?;

    let mut visitor = MacroVisitor {
        file: file_path,
        out: ParsedSource::default(),
    };
    visitor.visit_file(&syntax_tree);
    Ok(visitor.out)
}

/// What kind of failure a [`ParseError`] represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ParseErrorKind {
    /// The file could not be read from disk.
    Io,
    /// The file is not valid Rust.
    Rust,
    /// Markup inside a supported macro could not be parsed.
    Markup,
}

/// A file, or a macro within it, that could not be read or parsed.
///
/// The affected markup was **not** linted, so callers should treat parse
/// errors as failures rather than as clean results.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ParseError {
    /// What went wrong.
    pub kind: ParseErrorKind,
    /// The file the error occurred in.
    pub file: String,
    /// The line the error was found on (1-based), if known.
    pub line: Option<usize>,
    /// Human-readable description of the error.
    pub message: String,
}

impl ParseError {
    pub(crate) fn new(kind: ParseErrorKind, file: &str, message: impl Into<String>) -> Self {
        Self {
            kind,
            file: file.to_string(),
            line: None,
            message: message.into(),
        }
    }

    pub(crate) fn at_line(mut self, line: usize) -> Self {
        self.line = (line > 0).then_some(line);
        self
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let what = match self.kind {
            ParseErrorKind::Io => "Failed to read",
            ParseErrorKind::Rust => "Failed to parse",
            ParseErrorKind::Markup => "Invalid markup in",
        };
        match self.line {
            Some(line) => write!(f, "{} {}:{}: {}", what, self.file, line, self.message),
            None => write!(f, "{} {}: {}", what, self.file, self.message),
        }
    }
}

impl std::error::Error for ParseError {}

struct MacroVisitor<'a> {
    file: &'a str,
    out: ParsedSource,
}

impl<'ast> Visit<'ast> for MacroVisitor<'_> {
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        let name = mac
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        process_macro(&name, mac.tokens.clone(), self.file, &mut self.out);
        syn::visit::visit_macro(self, mac);
    }
}

/// Parse the markup in one macro invocation (if it is a markup macro), then
/// look for markup macros nested anywhere inside its tokens.
fn process_macro(name: &str, tokens: TokenStream, file: &str, out: &mut ParsedSource) {
    if MARKUP_MACROS.contains(&name) {
        let body = strip_leading_context(tokens.clone());
        // Yew also allows a root-level `if cond { <p/> }` in html!.
        let yew_root_if = matches!(name, "html" | "html_nested") && starts_with_ident(&body, "if");
        let parsed = if starts_with_lt(&body) || yew_root_if {
            Some(jsx::parse(body))
        } else if name == "rsx" && !body.is_empty() {
            Some(dioxus::parse(body))
        } else {
            None
        };
        if let Some((nodes, errors)) = parsed {
            markup::lower(&nodes, file, &mut out.elements);
            out.errors.extend(errors.into_iter().map(|e| {
                let line = e.span().start().line;
                ParseError::new(ParseErrorKind::Markup, file, format!("in `{name}!`: {e}"))
                    .at_line(line)
            }));
        }
    }
    for (nested_name, nested_tokens) in nested_macros(tokens) {
        process_macro(&nested_name, nested_tokens, file, out);
    }
}

/// Old Leptos versions take a reactive scope first: `view! { cx, <div/> }`.
fn strip_leading_context(tokens: TokenStream) -> TokenStream {
    let trees: Vec<TokenTree> = tokens.clone().into_iter().collect();
    match trees.as_slice() {
        [TokenTree::Ident(_), TokenTree::Punct(p), rest @ ..]
            if p.as_char() == ',' && rest.first().is_some_and(is_lt) =>
        {
            rest.iter().cloned().collect()
        }
        _ => tokens,
    }
}

fn is_lt(tt: &TokenTree) -> bool {
    matches!(tt, TokenTree::Punct(p) if p.as_char() == '<')
}

fn starts_with_ident(tokens: &TokenStream, word: &str) -> bool {
    matches!(tokens.clone().into_iter().next(), Some(TokenTree::Ident(i)) if i == word)
}

fn starts_with_lt(tokens: &TokenStream) -> bool {
    tokens.clone().into_iter().next().is_some_and(|t| is_lt(&t))
}

/// Find `name!(…)` invocations of markup macros anywhere in `tokens`,
/// without descending into the invocations themselves.
fn nested_macros(tokens: TokenStream) -> Vec<(String, TokenStream)> {
    let mut found = Vec::new();
    collect_nested(tokens, &mut found);
    found
}

fn collect_nested(tokens: TokenStream, found: &mut Vec<(String, TokenStream)>) {
    let trees: Vec<TokenTree> = tokens.into_iter().collect();
    let mut i = 0;
    while i < trees.len() {
        if let (
            TokenTree::Ident(ident),
            Some(TokenTree::Punct(bang)),
            Some(TokenTree::Group(group)),
        ) = (&trees[i], trees.get(i + 1), trees.get(i + 2))
            && bang.as_char() == '!'
            && group.delimiter() != Delimiter::None
        {
            let name = ident.to_string();
            if MARKUP_MACROS.contains(&name.as_str()) {
                found.push((name, group.stream()));
            } else {
                collect_nested(group.stream(), found);
            }
            i += 3;
            continue;
        }
        if let TokenTree::Group(group) = &trees[i] {
            collect_nested(group.stream(), found);
        }
        i += 1;
    }
}
