//! Parser for Rust source files and HTML-like macro content.
//!
//! Finds `html!`, `view!`, and `rsx!` macro invocations in Rust code, then
//! parses the HTML-like token streams within to extract elements and attributes.

use std::path::Path;
use syn::{spanned::Spanned, visit::Visit};

use crate::dom::{AttributeName, Role, Tag};
use rstml::node::{Node, NodeAttribute};

/// Represents an HTML element found in a macro invocation.
#[derive(Debug, Clone, serde::Serialize)]
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
    /// Column number in the source file (0-based).
    pub column: usize,
    /// The source file path.
    pub file: String,
}

impl HtmlElement {
    /// The effective role: explicit `role` attribute takes precedence,
    /// falling back to the tag's implicit role.
    pub fn role(&self) -> Option<Role> {
        self.attributes
            .iter()
            .find_map(|attr| {
                if attr.name == AttributeName::Role {
                    attr.value
                        .as_ref()
                        .and_then(|v| v.as_static())
                        .and_then(Role::from_str)
                } else {
                    None
                }
            })
            .or_else(|| self.tag.implicit_role())
    }

    /// Whether the element is focusable (natively interactive or has tabindex >= 0).
    pub fn is_focusable(&self) -> bool {
        self.tag.is_interactive()
            || self.attributes.iter().any(|a| {
                a.name == AttributeName::TabIndex
                    && match &a.value {
                        Some(AttrValue::Static(v)) => v.parse::<i32>().map_or(false, |i| i >= 0),
                        _ => true, // dynamic value; assume possibly focusable
                    }
            })
    }

    /// Whether there is an explicit event handler on this element.
    pub fn has_event_handler(&self) -> bool {
        self.attributes.iter().any(|a| {
            matches!(
                a.name,
                AttributeName::OnClick
                    | AttributeName::OnKeyDown
                    | AttributeName::OnKeyUp
                    | AttributeName::OnKeyPress
                    | AttributeName::OnMouseOver
                    | AttributeName::OnMouseOut
            )
        })
    }
}

/// Represents an attribute on an HTML element.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HtmlAttribute {
    /// Attribute name (e.g., "aria-label", "class", "role").
    pub name: AttributeName,
    /// Attribute value, if present.
    pub value: Option<AttrValue>,
    /// Line number in the source file (1-based).
    pub line: usize,
    /// Column number in the source file (0-based).
    pub column: usize,
}

/// Represents the value of an HTML attribute.
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub enum AttrValue {
    /// A static string literal value (contents without quotes).
    Static(String),
    /// A dynamic expression (e.g., `{some_variable}`) — cannot be checked statically.
    Dynamic,
}

impl AttrValue {
    /// Get the static string value, if available.
    pub fn as_static(&self) -> Option<&str> {
        match self {
            AttrValue::Static(s) => Some(s),
            AttrValue::Dynamic => None,
        }
    }
}

/// Normalize a path to use forward slashes consistently.
fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Parse a Rust source file and extract all HTML elements from supported macros.
///
/// Performs a cheap pre-filter before doing the expensive `syn` parse.
pub fn parse_file(path: &Path) -> Result<Vec<HtmlElement>, ParseError> {
    let file_path = normalize_path(path);
    let source = std::fs::read_to_string(path)
        .map_err(|e| ParseError::IoError(file_path.clone(), e.to_string()))?;

    parse_source(&source, &file_path)
}

/// Parse Rust source code and extract HTML elements from supported macros.
pub fn parse_source(source: &str, file_path: &str) -> Result<Vec<HtmlElement>, ParseError> {
    let syntax_tree = syn::parse_file(source)
        .map_err(|e| ParseError::SynError(file_path.to_string(), e.to_string()))?;

    let mut visitor = MacroVisitor {
        elements: Vec::new(),
        file_path: file_path.to_string(),
        rstml_errors: Vec::new(),
    };

    visitor.visit_file(&syntax_tree);

    // If no elements were found but rstml reported errors, surface them.
    if visitor.elements.is_empty() && !visitor.rstml_errors.is_empty() {
        return Err(ParseError::RstmlError(
            file_path.to_string(),
            visitor.rstml_errors.join("; "),
        ));
    }

    Ok(visitor.elements)
}

/// Errors that can occur during parsing.
#[derive(Debug, Clone, serde::Serialize)]
pub enum ParseError {
    IoError(String, String),
    SynError(String, String),
    /// RSX/HTML content inside a macro could not be parsed.
    RstmlError(String, String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IoError(path, err) => write!(f, "Failed to read {}: {}", path, err),
            ParseError::SynError(path, err) => write!(f, "Failed to parse {}: {}", path, err),
            ParseError::RstmlError(path, err) => {
                write!(f, "Invalid RSX in {}: {}", path, err)
            }
        }
    }
}

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
                if let Some(tag) = Tag::from_str(&node_element.name().to_string()) {
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
                                let attr_key = keyed_attribute.key.to_string();
                                HtmlAttribute {
                                    name: AttributeName::from_str(&attr_key)
                                        .unwrap_or(AttributeName::Unknown(attr_key)),
                                    value: Some(
                                        keyed_attribute
                                            .value_literal_string()
                                            .map(AttrValue::Static)
                                            .unwrap_or(AttrValue::Dynamic),
                                    ),
                                    line: line_column.line,
                                    column: line_column.column,
                                }
                            })
                            .collect(),
                        is_self_closing: node_element.close_tag.is_none(),
                        has_children: !node_element.children.is_empty(),
                        line: line_column.line,
                        column: line_column.column,
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
        assert!(matches!(attr.value, Some(AttrValue::Dynamic)));
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
            matches!(attr.value, Some(AttrValue::Dynamic)),
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
            matches!(attr.value, Some(AttrValue::Dynamic)),
            "expected Dynamic, got {:?}",
            attr.value
        );
    }
}
