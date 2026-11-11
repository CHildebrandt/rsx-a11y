//! Parser for Rust source files and HTML-like macro content.
//!
//! Finds `html!`, `view!`, and `rsx!` macro invocations in Rust code, then
//! parses the HTML-like token streams within to extract elements and attributes.

use std::path::Path;
use syn::{spanned::Spanned, visit::Visit};

use crate::dom::{AttributeName, Role, Tag};
use rstml::node::{Node, NodeAttribute};

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

/// Normalize a path to use forward slashes consistently.
fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Parse a Rust source file and extract all HTML elements from supported macros.
pub fn parse_file(path: &Path) -> Result<Vec<HtmlElement>, ParseError> {
    let file_path = normalize_path(path);
    let source = std::fs::read_to_string(path)
        .map_err(|e| ParseError::new(ParseErrorKind::Io, &file_path, e.to_string()))?;

    parse_source(&source, &file_path)
}

/// Parse Rust source code and extract HTML elements from supported macros.
pub fn parse_source(source: &str, file_path: &str) -> Result<Vec<HtmlElement>, ParseError> {
    let syntax_tree = syn::parse_file(source)
        .map_err(|e| ParseError::new(ParseErrorKind::Rust, file_path, e.to_string()))?;

    let mut visitor = MacroVisitor {
        elements: Vec::new(),
        file_path: file_path.to_string(),
        rstml_errors: Vec::new(),
    };

    visitor.visit_file(&syntax_tree);

    // If no elements were found but rstml reported errors, surface them.
    if visitor.elements.is_empty() && !visitor.rstml_errors.is_empty() {
        return Err(ParseError::new(
            ParseErrorKind::Markup,
            file_path,
            visitor.rstml_errors.join("; "),
        ));
    }

    Ok(visitor.elements)
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

/// A file (or a macro within it) that could not be read or parsed.
///
/// Files with parse errors were **not** linted, so callers should treat them
/// as failures rather than as clean results.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ParseError {
    /// What went wrong.
    pub kind: ParseErrorKind,
    /// The file the error occurred in.
    pub file: String,
    /// Human-readable description of the error.
    pub message: String,
}

impl ParseError {
    pub(crate) fn new(kind: ParseErrorKind, file: &str, message: impl Into<String>) -> Self {
        Self {
            kind,
            file: file.to_string(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let what = match self.kind {
            ParseErrorKind::Io => "Failed to read",
            ParseErrorKind::Rust => "Failed to parse",
            ParseErrorKind::Markup => "Invalid markup in",
        };
        write!(f, "{} {}: {}", what, self.file, self.message)
    }
}

impl std::error::Error for ParseError {}

/// AST visitor that finds macro invocations.
struct MacroVisitor {
    elements: Vec<HtmlElement>,
    file_path: String,
    /// Errors from rstml when parsing macro token streams.
    rstml_errors: Vec<String>,
}

impl<'ast> Visit<'ast> for MacroVisitor {
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        match rstml::parse2(mac.tokens.clone()) {
            Ok(nodes) => {
                let mut elements = Vec::new();
                collect_elements_from_nodes(&mut elements, &nodes, &self.file_path);
                self.elements.append(&mut elements);
            }
            Err(err) => {
                self.rstml_errors.push(err.to_string());
            }
        }
        syn::visit::visit_macro(self, mac);
    }
}

/// Recursively collect HtmlElements from rstml nodes.
fn collect_elements_from_nodes(acc: &mut Vec<HtmlElement>, nodes: &[Node], file_path: &str) {
    for node in nodes {
        match node {
            Node::Element(node_element) => {
                if let Ok(tag) = node_element.name().to_string().parse::<Tag>() {
                    let line_column = node_element.name().span().start();
                    let element = HtmlElement {
                        tag,
                        attributes: node_element
                            .attributes()
                            .iter()
                            .filter_map(|attr| match attr {
                                NodeAttribute::Attribute(keyed_attribute) => Some(keyed_attribute),
                                NodeAttribute::Block(_) => None,
                            })
                            .map(|keyed_attribute| {
                                let line_column = keyed_attribute.key.span().start();
                                let value = if keyed_attribute.value().is_none() {
                                    AttrValue::Bare
                                } else {
                                    keyed_attribute
                                        .value_literal_string()
                                        .map_or(AttrValue::Dynamic, AttrValue::Static)
                                };
                                HtmlAttribute {
                                    name: AttributeName::parse(&keyed_attribute.key.to_string()),
                                    value,
                                    line: line_column.line,
                                    column: line_column.column + 1,
                                }
                            })
                            .collect(),
                        is_self_closing: node_element.close_tag.is_none(),
                        has_children: !node_element.children.is_empty(),
                        line: line_column.line,
                        column: line_column.column + 1,
                        file: file_path.to_string(),
                    };
                    acc.push(element);
                }
                // Recurse into children
                collect_elements_from_nodes(acc, &node_element.children, file_path);
            }
            Node::Fragment(fragment) => {
                collect_elements_from_nodes(acc, &fragment.children, file_path);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::Aria;

    fn parse_test(source: &str) -> Vec<HtmlElement> {
        parse_source(source, "test.rs").unwrap()
    }

    #[test]
    fn test_parse_simple_div() {
        let elements = parse_test(
            r#"
            fn component() {
                html! {
                    <div class="container">
                        <p>{"Hello"}</p>
                    </div>
                }
            }
        "#,
        );
        assert!(elements.iter().any(|e| e.tag == Tag::Div));
        assert!(elements.iter().any(|e| e.tag == Tag::P));
    }

    #[test]
    fn test_parse_self_closing_img() {
        let elements = parse_test(
            r#"
            fn component() {
                html! {
                    <img src="test.png" />
                }
            }
        "#,
        );
        let img = elements.iter().find(|e| e.tag == Tag::Img).unwrap();
        assert!(img.is_self_closing);
        assert!(img.attributes.iter().any(|a| a.name == AttributeName::Src));
    }

    #[test]
    fn test_parse_aria_attributes() {
        let elements = parse_test(
            r#"
            fn component() {
                html! {
                    <div aria-label="main content" aria-hidden="false" role="main">
                    </div>
                }
            }
        "#,
        );
        let div = elements.iter().find(|e| e.tag == Tag::Div).unwrap();
        assert!(
            div.attributes
                .iter()
                .any(|a| a.name == AttributeName::Aria(Aria::Label))
        );
        assert!(
            div.attributes
                .iter()
                .any(|a| a.name == AttributeName::Aria(Aria::Hidden))
        );
        assert!(div.attributes.iter().any(|a| a.name == AttributeName::Role));
    }

    #[test]
    fn test_parse_leptos_view_macro() {
        let elements = parse_test(
            r#"
            fn component() {
                view! {
                    <button aria-pressed="true">{"Click"}</button>
                }
            }
        "#,
        );
        let btn = elements.iter().find(|e| e.tag == Tag::Button).unwrap();
        assert!(
            btn.attributes
                .iter()
                .any(|a| a.name == AttributeName::Aria(Aria::Pressed))
        );
    }

    #[test]
    fn test_parse_dynamic_value() {
        let elements = parse_test(
            r#"
            fn component() {
                html! {
                    <div aria-hidden={is_hidden}>
                    </div>
                }
            }
        "#,
        );
        let div = elements.iter().find(|e| e.tag == Tag::Div).unwrap();
        let attr = div
            .attributes
            .iter()
            .find(|a| a.name == AttributeName::Aria(Aria::Hidden))
            .unwrap();
        assert_eq!(attr.value, AttrValue::Dynamic);
    }

    #[test]
    fn test_parse_leptos_prefixed_attributes() {
        let elements = parse_test(
            r#"
            fn component() {
                view! {
                    <button on:click={handler} class:active={is_active}>{"Click"}</button>
                }
            }
        "#,
        );
        let btn = elements.iter().find(|e| e.tag == Tag::Button).unwrap();
        assert!(
            btn.attributes
                .iter()
                .any(|a| a.name == AttributeName::OnClick)
        );
        assert!(
            btn.attributes
                .iter()
                .any(|a| a.name == AttributeName::Unknown("class:active".into()))
        );
    }

    #[test]
    fn test_parse_fragment() {
        // Yew fragments: <> ... </>
        let elements = parse_test(
            r#"
            fn component() {
                html! {
                    <>
                        <div>{"Hello"}</div>
                    </>
                }
            }
        "#,
        );
        assert!(elements.iter().any(|e| e.tag == Tag::Div));
    }

    #[test]
    fn test_has_children() {
        let elements = parse_test(
            r#"
            fn component() {
                html! {
                    <div>
                        <span>{"text"}</span>
                    </div>
                }
            }
        "#,
        );
        let div = elements.iter().find(|e| e.tag == Tag::Div).unwrap();
        assert!(div.has_children);
    }

    #[test]
    fn test_closure_attr_value_is_dynamic() {
        // `move || if cond() { Some("page") } else { None }` must be parsed
        // as AttrValue::Dynamic rather than AttrValue::Static("move").
        let elements = parse_test(
            r#"
            fn component() {
                view! {
                    <a aria-current=move || if is_active() { Some("page") } else { None }>
                        {"Link"}
                    </a>
                }
            }
        "#,
        );
        let a = elements.iter().find(|e| e.tag == Tag::A).unwrap();
        let attr = a
            .attributes
            .iter()
            .find(|a| a.name == AttributeName::Aria(Aria::Current))
            .unwrap();
        assert!(
            attr.value == AttrValue::Dynamic,
            "expected Dynamic, got {:?}",
            attr.value
        );
    }

    #[test]
    fn test_if_expr_attr_value_is_dynamic() {
        // `if cond { "value" } else { "other" }` must be Dynamic.
        let elements = parse_test(
            r#"
            fn component() {
                view! {
                    <div class=if dark { "bg-dark" } else { "bg-light" }>
                    </div>
                }
            }
        "#,
        );
        let div = elements.iter().find(|e| e.tag == Tag::Div).unwrap();
        let attr = div
            .attributes
            .iter()
            .find(|a| a.name == AttributeName::Class)
            .unwrap();
        assert!(
            attr.value == AttrValue::Dynamic,
            "expected Dynamic, got {:?}",
            attr.value
        );
    }
}
