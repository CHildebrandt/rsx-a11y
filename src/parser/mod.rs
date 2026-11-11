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
use crate::suppress::Suppressions;

/// Macro names whose bodies contain markup.
const MARKUP_MACROS: &[&str] = &["html", "html_nested", "view", "rsx"];

/// Represents an HTML element found in a macro invocation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct HtmlElement {
    /// Element tag name (e.g., "div", "img", "button").
    pub tag: Tag,
    /// Attributes on the element.
    pub attributes: Vec<HtmlAttribute>,
    /// Whether the element has spread attributes (`{..attrs}` in Leptos,
    /// `..attrs` in Dioxus), so it may have attributes that are not listed.
    pub has_spread_attributes: bool,
    /// Whether the element is self-closing (e.g., `<img />`).
    pub is_self_closing: bool,
    /// Whether the element has child content (non-blank text, elements,
    /// components or expressions).
    pub has_children: bool,
    /// The static text content of the element and its descendants (including
    /// the `alt` text of descendant images), whitespace-normalized.
    pub text: String,
    /// Whether any descendant content is only known at runtime (expressions,
    /// interpolated strings, or components), so `text` may be incomplete.
    pub has_dynamic_content: bool,
    /// This element's position in the file's element list.
    pub index: usize,
    /// The [`index`](Self::index) of the nearest enclosing HTML element in
    /// the same macro, if any.
    pub parent: Option<usize>,
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

    /// Whether the attribute is present, or might be supplied by spread
    /// attributes. Use this before reporting an attribute as *missing*.
    pub fn may_have_attr(&self, name: &AttributeName) -> bool {
        self.has_spread_attributes || self.has_attr(name)
    }

    /// Whether any of the given attributes is present or might be spread in.
    pub fn may_have_any_attr(&self, names: &[AttributeName]) -> bool {
        self.has_spread_attributes || self.has_any_attr(names)
    }

    /// The static string value of attribute `name`, if present and static.
    pub fn static_attr(&self, name: &AttributeName) -> Option<&str> {
        self.attr(name).and_then(|a| a.value.as_static())
    }

    /// The lowercased static `type` attribute of an `<input>`, `"text"` if it
    /// is absent, or `None` if it is dynamic.
    pub fn input_type(&self) -> Option<String> {
        match self.attr(&AttributeName::Type) {
            None => Some("text".to_string()),
            Some(a) => a.value.as_static().map(|t| t.trim().to_ascii_lowercase()),
        }
    }

    /// Whether the element's `aria-hidden` is the static value `"true"`.
    pub fn is_aria_hidden(&self) -> bool {
        self.attr(&AttributeName::Aria(crate::dom::Aria::Hidden))
            .is_some_and(|a| a.value.is("true"))
    }

    /// The role set with an explicit, static `role` attribute: the first
    /// token that is a valid, non-abstract role (later tokens are fallbacks).
    pub fn explicit_role(&self) -> Option<Role> {
        self.static_attr(&AttributeName::Role)?
            .split_whitespace()
            .filter_map(|token| token.parse::<Role>().ok())
            .find(|role| !role.is_abstract())
    }

    /// The element's implicit role, taking its attributes into account
    /// (`<input type="checkbox">` is a checkbox, `<a>` without `href` has no
    /// role, `<img alt="">` is presentational, …).
    pub fn implicit_role(&self) -> Option<Role> {
        let has = |name: AttributeName| self.has_attr(&name);
        match self.tag {
            Tag::A | Tag::Area => has(AttributeName::Href).then_some(Role::Link),
            Tag::Img => {
                if self.static_attr(&AttributeName::Alt) == Some("") {
                    Some(Role::Presentation)
                } else {
                    Some(Role::Img)
                }
            }
            Tag::Input => match self.input_type()?.as_str() {
                "checkbox" => Some(Role::Checkbox),
                "radio" => Some(Role::Radio),
                "range" => Some(Role::Slider),
                "number" => Some(Role::SpinButton),
                "button" | "submit" | "reset" | "image" => Some(Role::Button),
                "search" if !has(AttributeName::List) => Some(Role::SearchBox),
                "text" | "email" | "tel" | "url" | "search" if has(AttributeName::List) => {
                    Some(Role::Combobox)
                }
                "text" | "email" | "tel" | "url" => Some(Role::TextBox),
                // hidden, password, file, color, date, … have no ARIA role.
                _ => None,
            },
            Tag::Select => {
                let size_gt_1 = self
                    .static_attr(&AttributeName::Size)
                    .and_then(|s| s.trim().parse::<u32>().ok())
                    .is_some_and(|s| s > 1);
                if has(AttributeName::Multiple) || size_gt_1 {
                    Some(Role::ListBox)
                } else {
                    Some(Role::Combobox)
                }
            }
            Tag::Section => self
                .has_any_attr(&[
                    AttributeName::Aria(crate::dom::Aria::Label),
                    AttributeName::Aria(crate::dom::Aria::LabelledBy),
                    AttributeName::Title,
                ])
                .then_some(Role::Region),
            tag => tag.implicit_role(),
        }
    }

    /// The effective role: an explicit `role` attribute takes precedence,
    /// falling back to the implicit role.
    pub fn role(&self) -> Option<Role> {
        self.explicit_role().or_else(|| self.implicit_role())
    }

    /// Whether the element is natively interactive, considering attributes
    /// (`<a>` needs an `href`, `<input type="hidden">` is not interactive).
    pub fn is_natively_interactive(&self) -> bool {
        match self.tag {
            Tag::A | Tag::Area => self.has_attr(&AttributeName::Href),
            Tag::Input => self.input_type().is_none_or(|t| t != "hidden"),
            Tag::Audio | Tag::Video => self.has_attr(&AttributeName::Unknown("controls".into())),
            tag => tag.is_interactive(),
        }
    }

    /// Whether the element can receive keyboard focus: an explicit
    /// `tabindex >= 0`, or natively interactive and not disabled.
    pub fn is_focusable(&self) -> bool {
        if let Some(tabindex) = self.attr(&AttributeName::TabIndex) {
            return match &tabindex.value {
                AttrValue::Static(v) => v.trim().parse::<i32>().is_ok_and(|i| i >= 0),
                // Dynamic value: assume it may be focusable.
                AttrValue::Dynamic | AttrValue::Bare => true,
            };
        }
        self.is_natively_interactive() && !self.has_attr(&AttributeName::Disabled)
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
    /// Markup macros that could not be parsed (elements in them are missing
    /// from `elements`, so the file was only partially linted), and malformed
    /// `rsx-a11y-ignore` comments.
    pub errors: Vec<ParseError>,
    /// `// rsx-a11y-ignore` directives found in the file.
    pub suppressions: Suppressions,
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

    let (suppressions, errors) = Suppressions::parse(source, file_path);
    visitor.out.suppressions = suppressions;
    visitor.out.errors.extend(errors);
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
    /// An `rsx-a11y-ignore` comment is malformed (e.g. names an unknown rule).
    Directive,
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
            ParseErrorKind::Directive => "Invalid ignore comment in",
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
