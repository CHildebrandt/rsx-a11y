//! Parser tests.

use super::*;
use crate::dom::Aria;

fn parse_test(source: &str) -> Vec<HtmlElement> {
    let parsed = parse_source(source, "test.rs").unwrap();
    assert!(
        parsed.errors.is_empty(),
        "parse errors: {:?}",
        parsed.errors
    );
    parsed.elements
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

fn tags(elements: &[HtmlElement]) -> Vec<Tag> {
    elements.iter().map(|e| e.tag).collect()
}

#[test]
fn test_positions_are_one_based_and_point_at_the_tag() {
    let elements = parse_test("fn c() {\n    html! { <img src=\"x.png\" /> }\n}");
    let img = &elements[0];
    assert_eq!((img.line, img.column), (2, 13), "`<` of `<img`");
    let src = &img.attributes[0];
    assert_eq!((src.line, src.column), (2, 18));
}

#[test]
fn test_valueless_attribute_is_bare() {
    let elements = parse_test(r#"fn c() { html! { <input autofocus /> } }"#);
    assert_eq!(elements[0].attributes[0].value, AttrValue::Bare);
}

#[test]
fn test_unquoted_literal_values_are_static() {
    let elements = parse_test(r#"fn c() { view! { <div tabindex=0 aria-hidden=true></div> } }"#);
    let div = &elements[0];
    assert_eq!(div.static_attr(&AttributeName::TabIndex), Some("0"));
    assert_eq!(
        div.static_attr(&AttributeName::Aria(Aria::Hidden)),
        Some("true")
    );
}

#[test]
fn test_yew_for_iterator_block_does_not_break_parsing() {
    let parsed = parse_source(
        r#"fn c() { html! { <ul>{ for items.iter().map(|i| html! { <li><img src={i} /></li> }) }</ul> } }"#,
        "t.rs",
    )
    .unwrap();
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    assert_eq!(tags(&parsed.elements), [Tag::Ul, Tag::Li, Tag::Img]);
}

#[test]
fn test_nested_macros_in_blocks_closures_and_attributes_are_found() {
    let elements = parse_test(
        r#"
        fn c() {
            view! {
                <Show when=move || open.get() fallback=|| view! { <p>"closed"</p> }>
                    {move || items.get().into_iter().map(|i| view! { <li>{i}</li> }).collect_view()}
                </Show>
            }
        }
        fn d() -> Vec<Html> { vec![html! { <span /> }] }
        "#,
    );
    let mut found = tags(&elements);
    found.sort_by_key(|t| t.to_string());
    assert_eq!(found, [Tag::Li, Tag::P, Tag::Span]);
}

#[test]
fn test_yew_if_block_with_markup() {
    let elements =
        parse_test(r#"fn c() { html! { <div> if open { <img src="x.png" /> } </div> } }"#);
    assert!(tags(&elements).contains(&Tag::Img));
}

#[test]
fn test_components_are_skipped_but_their_children_are_linted() {
    let elements = parse_test(r#"fn c() { view! { <A href="/x"><img src="y.png" /></A> } }"#);
    assert_eq!(tags(&elements), [Tag::Img], "<A> is a component, not <a>");
}

#[test]
fn test_old_leptos_scope_argument() {
    let elements = parse_test(r#"fn c(cx: Scope) { view! { cx, <img src="x.png" /> } }"#);
    assert_eq!(tags(&elements), [Tag::Img]);
}

#[test]
fn test_non_markup_macros_are_ignored() {
    let parsed = parse_source(
        r#"
        fn c() {
            assert!(1 < 2);
            let v = vec![1, 2, 3];
            println!("{v:?}");
            assert_eq!(v.len(), 3, "len {}", v.len());
            maud::html! { p.greeting { "Hi" } };
            sycamore::view! { div(class="x") { "Hi" } };
        }
        "#,
        "t.rs",
    )
    .unwrap();
    assert!(parsed.elements.is_empty());
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
}

#[test]
fn test_markup_error_is_reported_per_macro_with_line() {
    let parsed = parse_source(
        "fn a() { html! { <img src=\"ok.png\" alt=\"ok\" /> } }\nfn b() { html! { <div> </span> } }",
        "t.rs",
    )
    .unwrap();
    assert_eq!(
        tags(&parsed.elements)[0],
        Tag::Img,
        "valid macro still linted"
    );
    assert_eq!(parsed.errors.len(), 1);
    assert_eq!(parsed.errors[0].kind, ParseErrorKind::Markup);
    assert_eq!(parsed.errors[0].line, Some(2));
}

#[test]
fn test_invalid_rust_is_an_error() {
    let err = parse_source("fn c( {", "t.rs").unwrap_err();
    assert_eq!(err.kind, ParseErrorKind::Rust);
}

// --- Dioxus RSX ---

#[test]
fn test_dioxus_elements_attributes_and_text() {
    let elements = parse_test(
        r#"
        fn App() -> Element {
            rsx! {
                div { class: "x", id: "main",
                    img { src: "a.png" }
                    button { r#type: "button", aria_label: "Close", onclick: move |_| close(), "×" }
                    a { href: "/docs", "Docs" }
                    input { "aria-describedby": "hint", tabindex: 0, disabled: true }
                }
            }
        }
        "#,
    );
    assert_eq!(
        tags(&elements),
        [Tag::Div, Tag::Img, Tag::Button, Tag::A, Tag::Input]
    );
    let button = &elements[2];
    assert_eq!(button.static_attr(&AttributeName::Type), Some("button"));
    assert_eq!(
        button.static_attr(&AttributeName::Aria(Aria::Label)),
        Some("Close")
    );
    assert!(button.has_attr(&AttributeName::OnClick));
    assert!(button.has_children);
    let img = &elements[1];
    assert!(!img.has_children);
    assert_eq!((img.line, img.column), (5, 21));
    let input = &elements[4];
    assert_eq!(
        input.static_attr(&AttributeName::Aria(Aria::DescribedBy)),
        Some("hint")
    );
    assert_eq!(input.static_attr(&AttributeName::TabIndex), Some("0"));
}

#[test]
fn test_dioxus_control_flow_components_and_interpolation() {
    let elements = parse_test(
        r#"
        fn App() -> Element {
            let label = "x";
            rsx! {
                for item in items.iter() {
                    li { key: "{item.id}", "{item.name}" }
                }
                if show {
                    span { "shown" }
                } else if other {
                    p {}
                } else {
                    h1 { title: "{label}" }
                }
                Card { title: "x", img { src: "c.png" } }
                ui::Panel::<u8> { aside {} }
                {children}
                {rsx! { em { "nested" } }}
                div { class, onclick }
                div { ..attributes, "spread" }
            }
        }
        "#,
    );
    let found = tags(&elements);
    for tag in [
        Tag::Li,
        Tag::Span,
        Tag::P,
        Tag::H1,
        Tag::Img,
        Tag::Aside,
        Tag::Em,
    ] {
        assert!(found.contains(&tag), "missing {tag}: {found:?}");
    }
    let h1 = elements.iter().find(|e| e.tag == Tag::H1).unwrap();
    assert_eq!(
        h1.attributes[0].value,
        AttrValue::Dynamic,
        "interpolated string"
    );
    let li = elements.iter().find(|e| e.tag == Tag::Li).unwrap();
    assert!(li.has_children, "interpolated text is dynamic content");
}

#[test]
fn test_dioxus_syntax_error_is_reported() {
    let parsed = parse_source("fn a() { rsx! { div { class: } } }", "t.rs").unwrap();
    assert_eq!(parsed.errors.len(), 1);
    assert!(parsed.errors[0].message.contains("rsx!"));
}

#[test]
fn test_jsx_style_rsx_still_uses_jsx_parser() {
    let elements = parse_test(r#"fn c() { rsx! { <img src="x.png" /> } }"#);
    assert_eq!(tags(&elements), [Tag::Img]);
}

#[test]
fn test_yew_shorthand_attribute_and_leptos_spread() {
    let elements = parse_test(r#"fn c() { html! { <div {onclick} {..rest}></div> } }"#);
    let div = &elements[0];
    assert!(
        div.has_attr(&AttributeName::OnClick),
        "{:?}",
        div.attributes
    );
    assert!(div.has_spread_attributes);
    let plain = parse_test(r#"fn c() { html! { <div {onclick}></div> } }"#);
    assert!(!plain[0].has_spread_attributes);
}

#[test]
fn test_qualified_markup_macro_is_parsed() {
    let elements = parse_test(r#"fn c() { leptos::view! { <img src="x.png" /> } }"#);
    assert_eq!(tags(&elements), [Tag::Img]);
}

// --- Review regressions (#10) ---

#[test]
fn test_negative_numeric_literals_are_static() {
    let dioxus = parse_test(r#"fn A() -> Element { rsx! { div { tabindex: -1 } } }"#);
    assert_eq!(dioxus[0].static_attr(&AttributeName::TabIndex), Some("-1"));
    let leptos = parse_test(r#"fn c() { view! { <div tabindex=-1></div> } }"#);
    assert_eq!(leptos[0].static_attr(&AttributeName::TabIndex), Some("-1"));
}

#[test]
fn test_errors_inside_markup_blocks_are_reported() {
    let parsed = parse_source(
        r#"fn c() { html! { <div> if show { <img></span> } </div> } }"#,
        "t.rs",
    )
    .unwrap();
    assert!(!parsed.errors.is_empty(), "malformed markup in an if-block");
}

#[test]
fn test_yew_root_if_is_parsed() {
    let elements = parse_test(
        r#"fn c() { html! { if show { <img src="x.png" /> } else { <p>{"none"}</p> } } }"#,
    );
    assert_eq!(tags(&elements), [Tag::Img, Tag::P]);
}

#[test]
fn test_markup_after_leading_text_is_parsed() {
    let elements = parse_test(
        r#"fn c() {
            view! { "Intro" <img src="x.png" /> };
            view! { {move || count.get()} " items" <p /> };
        }"#,
    );
    assert_eq!(tags(&elements), [Tag::Img, Tag::P]);
    // Leading text does not make other macro syntaxes look like markup.
    let elements = parse_test(
        r#"fn c() {
            maud::html! { "Hi " strong { "there" } };
            sycamore::view! { "Hi " (name) };
        }"#,
    );
    assert!(elements.is_empty(), "{elements:?}");
}

#[test]
fn test_nested_yew_if_blocks_are_parsed() {
    let elements = parse_test(
        r#"fn c() {
            html! { if outer { if inner { <img src="x.png" /> } } };
            html! { <div> if a { if b { <p /> } else { <span /> } } </div> };
        }"#,
    );
    assert_eq!(tags(&elements), [Tag::Img, Tag::Div, Tag::P, Tag::Span]);
    // Markup errors inside the nested branch are still reported.
    let parsed = parse_source(r#"fn c() { html! { if a { if b { <p> } } } }"#, "test.rs").unwrap();
    assert!(!parsed.errors.is_empty());
}

#[test]
fn test_nested_macro_elements_are_in_source_order() {
    let elements =
        parse_test(r#"fn c() { html! { <div>{html! { <img src="x.png" /> }}<p /></div> } }"#);
    assert_eq!(tags(&elements), [Tag::Div, Tag::Img, Tag::P]);
    for (i, el) in elements.iter().enumerate() {
        assert_eq!(el.index, i);
    }
    // Parents follow the renumbering; the nested macro's element has none.
    assert_eq!(elements[1].parent, None);
    assert_eq!(elements[2].parent, Some(0));
}
