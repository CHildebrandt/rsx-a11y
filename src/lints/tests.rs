//! Unit tests for the individual lint rules.

use super::*;
use crate::parser;

fn lint_source(source: &str) -> Vec<LintDiagnostic> {
    let elements = parser::parse_source(source, "test.rs").unwrap();
    run_all_lints(&elements).collect()
}

fn has_lint(diags: &[LintDiagnostic], id: Rule) -> bool {
    diags.iter().any(|d| d.rule == id)
}

#[test]
fn test_invalid_aria_attribute() {
    let diags = lint_source(r#"fn c() { html! { <div aria-foo="bar"></div> } }"#);
    assert!(has_lint(&diags, Rule::AriaProps));
}

#[test]
fn test_valid_aria_attribute() {
    let diags = lint_source(r#"fn c() { html! { <div aria-label="hello"></div> } }"#);
    assert!(!has_lint(&diags, Rule::AriaProps));
}

#[test]
fn test_invalid_aria_value() {
    let diags = lint_source(r#"fn c() { html! { <div aria-hidden="yes"></div> } }"#);
    assert!(has_lint(&diags, Rule::AriaProptypes));
}

#[test]
fn test_valid_aria_value() {
    let diags = lint_source(r#"fn c() { html! { <div aria-hidden="true"></div> } }"#);
    assert!(!has_lint(&diags, Rule::AriaProptypes));
}

#[test]
fn test_invalid_role() {
    let diags = lint_source(r#"fn c() { html! { <div role="banana"></div> } }"#);
    assert!(has_lint(&diags, Rule::AriaRole));
}

#[test]
fn test_valid_role() {
    let diags = lint_source(r#"fn c() { html! { <div role="navigation"></div> } }"#);
    assert!(!has_lint(&diags, Rule::AriaRole));
}

#[test]
fn test_abstract_role() {
    let diags = lint_source(r#"fn c() { html! { <div role="widget"></div> } }"#);
    assert!(has_lint(&diags, Rule::AriaRole));
}

#[test]
fn test_redundant_role() {
    let diags = lint_source(r#"fn c() { html! { <button role="button">{"Click"}</button> } }"#);
    assert!(has_lint(&diags, Rule::NoRedundantRoles));
}

#[test]
fn test_missing_alt_text() {
    let diags = lint_source(r#"fn c() { html! { <img src="test.png" /> } }"#);
    assert!(has_lint(&diags, Rule::AltText));
}

#[test]
fn test_img_with_alt() {
    let diags = lint_source(
        r#"fn c() { html! { <img src="test.png" alt="A test image description" /> } }"#,
    );
    assert!(!has_lint(&diags, Rule::AltText));
}

#[test]
fn test_img_presentation_role() {
    let diags = lint_source(r#"fn c() { html! { <img src="test.png" role="presentation" /> } }"#);
    assert!(!has_lint(&diags, Rule::AltText));
}

#[test]
fn test_no_access_key() {
    let diags = lint_source(r#"fn c() { html! { <button accesskey="s">{"Save"}</button> } }"#);
    assert!(has_lint(&diags, Rule::NoAccessKey));
}

#[test]
fn test_no_autofocus() {
    let diags = lint_source(r#"fn c() { html! { <input autofocus /> } }"#);
    assert!(has_lint(&diags, Rule::NoAutofocus));
}

#[test]
fn test_no_distracting_elements() {
    let diags = lint_source(r#"fn c() { html! { <marquee>{"Scrolling text"}</marquee> } }"#);
    assert!(has_lint(&diags, Rule::NoDistractingElements));
}

#[test]
fn test_anchor_invalid_href() {
    let diags = lint_source(r##"fn c() { html! { <a href="#">{"Link"}</a> } }"##);
    assert!(has_lint(&diags, Rule::AnchorIsValid));
}

#[test]
fn test_anchor_valid_href() {
    let diags = lint_source(r#"fn c() { html! { <a href="/about">{"About"}</a> } }"#);
    assert!(!has_lint(&diags, Rule::AnchorIsValid));
}

#[test]
fn test_no_redundant_alt() {
    let diags = lint_source(r#"fn c() { html! { <img src="test.png" alt="image of a cat" /> } }"#);
    assert!(has_lint(&diags, Rule::ImgRedundantAlt));
}

#[test]
fn test_positive_tabindex() {
    let diags = lint_source(r#"fn c() { html! { <div tabindex="5"></div> } }"#);
    assert!(has_lint(&diags, Rule::TabindexNoPositive));
}

#[test]
fn test_click_without_keyboard() {
    let diags = lint_source(r#"fn c() { html! { <div onclick={handler}></div> } }"#);
    assert!(has_lint(&diags, Rule::ClickEventsHaveKeyEvents));
}

#[test]
fn test_click_without_keyboard_with_role_button() {
    let diags = lint_source(r#"fn c() { html! { <div role="button" onclick={handler}></div> } }"#);
    assert!(has_lint(&diags, Rule::ClickEventsHaveKeyEvents));
}

#[test]
fn test_click_on_button_ok() {
    let diags = lint_source(r#"fn c() { html! { <button onclick={handler}>{"Click"}</button> } }"#);
    assert!(!has_lint(&diags, Rule::ClickEventsHaveKeyEvents));
}

#[test]
fn test_leptos_view_macro() {
    let diags = lint_source(r#"fn c() { view! { <img src="test.png" /> } }"#);
    assert!(has_lint(&diags, Rule::AltText));
}

#[test]
fn test_iframe_missing_title() {
    let diags = lint_source(r#"fn c() { html! { <iframe src="https://example.com"></iframe> } }"#);
    assert!(has_lint(&diags, Rule::IframeHasTitle));
}

#[test]
fn test_aria_on_unsupported_element() {
    let diags = lint_source(r#"fn c() { html! { <meta aria-label="test" /> } }"#);
    assert!(has_lint(&diags, Rule::AriaUnsupportedElements));
}

// --- AnchorAmbiguousText ---

#[test]
fn test_anchor_ambiguous_text_click_here() {
    let diags =
        lint_source(r#"fn c() { html! { <a href="/x" aria-label="click here">{"x"}</a> } }"#);
    assert!(has_lint(&diags, Rule::AnchorAmbiguousText));
}

#[test]
fn test_anchor_ambiguous_text_ok() {
    let diags = lint_source(
        r#"fn c() { html! { <a href="/about" aria-label="Read about our company">{"x"}</a> } }"#,
    );
    assert!(!has_lint(&diags, Rule::AnchorAmbiguousText));
}

// --- AnchorHasContent ---

#[test]
fn test_anchor_has_content_empty() {
    let diags = lint_source(r#"fn c() { html! { <a href="/about"></a> } }"#);
    assert!(has_lint(&diags, Rule::AnchorHasContent));
}

#[test]
fn test_anchor_has_content_with_children() {
    let diags = lint_source(r#"fn c() { html! { <a href="/about">{"About"}</a> } }"#);
    assert!(!has_lint(&diags, Rule::AnchorHasContent));
}

#[test]
fn test_anchor_has_content_with_aria_label() {
    let diags = lint_source(r#"fn c() { html! { <a href="/about" aria-label="About us"></a> } }"#);
    assert!(!has_lint(&diags, Rule::AnchorHasContent));
}

// --- AriaActivedescendantHasTabindex ---

#[test]
fn test_activedescendant_without_tabindex() {
    let diags = lint_source(r#"fn c() { html! { <div aria-activedescendant="item1"></div> } }"#);
    assert!(has_lint(&diags, Rule::AriaActivedescendantHasTabindex));
}

#[test]
fn test_activedescendant_with_tabindex() {
    let diags = lint_source(
        r#"fn c() { html! { <div aria-activedescendant="item1" tabindex="0"></div> } }"#,
    );
    assert!(!has_lint(&diags, Rule::AriaActivedescendantHasTabindex));
}

#[test]
fn test_activedescendant_on_interactive_ok() {
    let diags = lint_source(r#"fn c() { html! { <input aria-activedescendant="item1" /> } }"#);
    assert!(!has_lint(&diags, Rule::AriaActivedescendantHasTabindex));
}

// --- AutocompleteValid ---

#[test]
fn test_autocomplete_valid() {
    let diags = lint_source(r#"fn c() { html! { <input autocomplete="email" /> } }"#);
    assert!(!has_lint(&diags, Rule::AutocompleteValid));
}

#[test]
fn test_autocomplete_invalid() {
    let diags = lint_source(r#"fn c() { html! { <input autocomplete="banana" /> } }"#);
    assert!(has_lint(&diags, Rule::AutocompleteValid));
}

// --- ControlHasAssociatedLabel ---

#[test]
fn test_control_without_label() {
    let diags = lint_source(r#"fn c() { html! { <input /> } }"#);
    assert!(has_lint(&diags, Rule::ControlHasAssociatedLabel));
}

#[test]
fn test_control_with_aria_label() {
    let diags = lint_source(r#"fn c() { html! { <input aria-label="Search" /> } }"#);
    assert!(!has_lint(&diags, Rule::ControlHasAssociatedLabel));
}

#[test]
fn test_button_with_children_ok() {
    let diags = lint_source(r#"fn c() { html! { <button>{"Submit"}</button> } }"#);
    assert!(!has_lint(&diags, Rule::ControlHasAssociatedLabel));
}

// --- HeadingHasContent ---

#[test]
fn test_heading_empty() {
    let diags = lint_source(r#"fn c() { html! { <h2></h2> } }"#);
    assert!(has_lint(&diags, Rule::HeadingHasContent));
}

#[test]
fn test_heading_with_content() {
    let diags = lint_source(r#"fn c() { html! { <h2>{"Title"}</h2> } }"#);
    assert!(!has_lint(&diags, Rule::HeadingHasContent));
}

#[test]
fn test_heading_with_aria_label() {
    let diags = lint_source(r#"fn c() { html! { <h2 aria-label="Title"></h2> } }"#);
    assert!(!has_lint(&diags, Rule::HeadingHasContent));
}

// --- HtmlHasLang ---

#[test]
fn test_html_missing_lang() {
    let diags = lint_source(r#"fn c() { html! { <html></html> } }"#);
    assert!(has_lint(&diags, Rule::HtmlHasLang));
}

#[test]
fn test_html_with_lang() {
    let diags = lint_source(r#"fn c() { html! { <html lang="en"></html> } }"#);
    assert!(!has_lint(&diags, Rule::HtmlHasLang));
}

// --- InteractiveSupportsFocus ---

#[test]
fn test_interactive_role_without_tabindex() {
    let diags = lint_source(r#"fn c() { html! { <div role="button" onclick={handler}></div> } }"#);
    assert!(has_lint(&diags, Rule::InteractiveSupportsFocus));
}

#[test]
fn test_interactive_role_with_tabindex() {
    let diags = lint_source(
        r#"fn c() { html! { <div role="button" onclick={handler} tabindex="0"></div> } }"#,
    );
    assert!(!has_lint(&diags, Rule::InteractiveSupportsFocus));
}

// --- LabelHasAssociatedControl ---

#[test]
fn test_label_without_for_or_children() {
    let diags = lint_source(r#"fn c() { html! { <label></label> } }"#);
    assert!(has_lint(&diags, Rule::LabelHasAssociatedControl));
}

#[test]
fn test_label_with_for() {
    let diags = lint_source(r#"fn c() { html! { <label for="email">{"Email"}</label> } }"#);
    assert!(!has_lint(&diags, Rule::LabelHasAssociatedControl));
}

// --- Lang ---

#[test]
fn test_lang_invalid_value() {
    let diags = lint_source(r#"fn c() { html! { <html lang="123"></html> } }"#);
    assert!(has_lint(&diags, Rule::Lang));
}

#[test]
fn test_lang_valid_value() {
    let diags = lint_source(r#"fn c() { html! { <html lang="en-US"></html> } }"#);
    assert!(!has_lint(&diags, Rule::Lang));
}

// --- MediaHasCaption ---

#[test]
fn test_video_without_caption() {
    let diags = lint_source(r#"fn c() { html! { <video src="v.mp4"></video> } }"#);
    assert!(has_lint(&diags, Rule::MediaHasCaption));
}

#[test]
fn test_audio_without_caption() {
    let diags = lint_source(r#"fn c() { html! { <audio src="a.mp3"></audio> } }"#);
    assert!(has_lint(&diags, Rule::MediaHasCaption));
}

#[test]
fn test_video_with_aria_label_ok() {
    let diags =
        lint_source(r#"fn c() { html! { <video src="v.mp4" aria-label="Tutorial"></video> } }"#);
    assert!(!has_lint(&diags, Rule::MediaHasCaption));
}

// --- MouseEventsHaveKeyEvents ---

#[test]
fn test_mouseover_without_focus() {
    let diags = lint_source(r#"fn c() { html! { <div onmouseover={handler}></div> } }"#);
    assert!(has_lint(&diags, Rule::MouseEventsHaveKeyEvents));
}

#[test]
fn test_mouseover_with_focus_ok() {
    let diags =
        lint_source(r#"fn c() { html! { <div onmouseover={handler} onfocus={handler}></div> } }"#);
    assert!(!has_lint(&diags, Rule::MouseEventsHaveKeyEvents));
}

#[test]
fn test_mouseout_without_blur() {
    let diags = lint_source(r#"fn c() { html! { <div onmouseout={handler}></div> } }"#);
    assert!(has_lint(&diags, Rule::MouseEventsHaveKeyEvents));
}

// --- NoAriaHiddenOnFocusable ---

#[test]
fn test_aria_hidden_on_button() {
    let diags = lint_source(r#"fn c() { html! { <button aria-hidden="true">{"x"}</button> } }"#);
    assert!(has_lint(&diags, Rule::NoAriaHiddenOnFocusable));
}

#[test]
fn test_aria_hidden_on_nonfocusable_ok() {
    let diags = lint_source(r#"fn c() { html! { <div aria-hidden="true"></div> } }"#);
    assert!(!has_lint(&diags, Rule::NoAriaHiddenOnFocusable));
}

// --- NoInteractiveElementToNoninteractiveRole ---

#[test]
fn test_button_with_noninteractive_role() {
    let diags = lint_source(r#"fn c() { html! { <button role="article">{"x"}</button> } }"#);
    assert!(has_lint(
        &diags,
        Rule::NoInteractiveElementToNoninteractiveRole
    ));
}

#[test]
fn test_button_with_interactive_role_ok() {
    let diags = lint_source(r#"fn c() { html! { <button role="tab">{"x"}</button> } }"#);
    assert!(!has_lint(
        &diags,
        Rule::NoInteractiveElementToNoninteractiveRole
    ));
}

// --- NoNoninteractiveElementInteractions ---

#[test]
fn test_article_with_click_handler() {
    // <article> has implicit role "article" which is non-interactive
    let diags = lint_source(r#"fn c() { html! { <article onclick={handler}></article> } }"#);
    assert!(has_lint(&diags, Rule::NoNoninteractiveElementInteractions));
}

#[test]
fn test_article_with_interactive_role_and_click_ok() {
    let diags =
        lint_source(r#"fn c() { html! { <article role="button" onclick={handler}></article> } }"#);
    assert!(!has_lint(&diags, Rule::NoNoninteractiveElementInteractions));
}

// --- NoNoninteractiveElementToInteractiveRole ---

#[test]
fn test_div_with_interactive_role() {
    let diags = lint_source(r#"fn c() { html! { <div role="button"></div> } }"#);
    assert!(has_lint(
        &diags,
        Rule::NoNoninteractiveElementToInteractiveRole
    ));
}

#[test]
fn test_div_with_noninteractive_role_ok() {
    let diags = lint_source(r#"fn c() { html! { <div role="article"></div> } }"#);
    assert!(!has_lint(
        &diags,
        Rule::NoNoninteractiveElementToInteractiveRole
    ));
}

// --- NoNoninteractiveTabindex ---

#[test]
fn test_span_with_tabindex_zero() {
    let diags = lint_source(r#"fn c() { html! { <span tabindex="0"></span> } }"#);
    assert!(has_lint(&diags, Rule::NoNoninteractiveTabindex));
}

#[test]
fn test_span_with_tabindex_negative_ok() {
    let diags = lint_source(r#"fn c() { html! { <span tabindex="-1"></span> } }"#);
    assert!(!has_lint(&diags, Rule::NoNoninteractiveTabindex));
}

// --- NoStaticElementInteractions ---

#[test]
fn test_div_with_handler_no_role() {
    let diags = lint_source(r#"fn c() { html! { <div onclick={handler}></div> } }"#);
    assert!(has_lint(&diags, Rule::NoStaticElementInteractions));
}

#[test]
fn test_div_with_handler_and_role_ok() {
    let diags = lint_source(r#"fn c() { html! { <div role="button" onclick={handler}></div> } }"#);
    assert!(!has_lint(&diags, Rule::NoStaticElementInteractions));
}

// --- PreferTagOverRole ---

#[test]
fn test_prefer_tag_over_role_button() {
    let diags = lint_source(r#"fn c() { html! { <div role="button"></div> } }"#);
    assert!(has_lint(&diags, Rule::PreferTagOverRole));
}

#[test]
fn test_prefer_tag_same_element_ok() {
    // <button role="button"> is handled by NoRedundantRoles, not PreferTagOverRole
    let diags = lint_source(r#"fn c() { html! { <button role="button">{"x"}</button> } }"#);
    assert!(!has_lint(&diags, Rule::PreferTagOverRole));
}

// --- RoleHasRequiredAriaProps ---

#[test]
fn test_checkbox_missing_checked() {
    let diags = lint_source(r#"fn c() { html! { <div role="checkbox"></div> } }"#);
    assert!(has_lint(&diags, Rule::RoleHasRequiredAriaProps));
}

#[test]
fn test_checkbox_with_checked_ok() {
    let diags =
        lint_source(r#"fn c() { html! { <div role="checkbox" aria-checked="false"></div> } }"#);
    assert!(!has_lint(&diags, Rule::RoleHasRequiredAriaProps));
}

// --- RoleSupportsAriaProps ---

#[test]
fn test_unsupported_aria_prop_for_role() {
    // aria-checked is not supported on the "alert" role
    let diags = lint_source(r#"fn c() { html! { <div role="alert" aria-checked="true"></div> } }"#);
    assert!(has_lint(&diags, Rule::RoleSupportsAriaProps));
}

#[test]
fn test_supported_aria_prop_for_role_ok() {
    // aria-label is a global prop, supported everywhere
    let diags =
        lint_source(r#"fn c() { html! { <div role="alert" aria-label="Warning"></div> } }"#);
    assert!(!has_lint(&diags, Rule::RoleSupportsAriaProps));
}

// --- Scope ---

#[test]
fn test_scope_on_td() {
    let diags = lint_source(r#"fn c() { html! { <td scope="row">{"x"}</td> } }"#);
    assert!(has_lint(&diags, Rule::Scope));
}

#[test]
fn test_scope_on_th_ok() {
    let diags = lint_source(r#"fn c() { html! { <th scope="col">{"Header"}</th> } }"#);
    assert!(!has_lint(&diags, Rule::Scope));
}

// --- Multiple diagnostics per element ---

#[test]
fn test_every_violation_on_an_element_is_reported() {
    let diags = lint_source(
        r#"fn c() { html! { <div aria-foo="1" aria-bar="2" aria-hidden="yes" aria-busy="nope" onmouseover={a} onmouseout={b}></div> } }"#,
    );
    let count = |rule| diags.iter().filter(|d| d.rule == rule).count();
    assert_eq!(count(Rule::AriaProps), 2);
    assert_eq!(count(Rule::AriaProptypes), 2);
    assert_eq!(count(Rule::MouseEventsHaveKeyEvents), 2);
}

#[test]
fn test_rule_names_round_trip() {
    for rule in Rule::all() {
        assert_eq!(rule.as_str().parse::<Rule>(), Ok(rule));
        assert_eq!(
            serde_json::to_string(&rule).unwrap(),
            format!("\"{}\"", rule.as_str())
        );
    }
    assert_eq!("alt-text".parse::<Rule>(), Ok(Rule::AltText));
    assert!(Rule::all().all(|r| !r.description().starts_with(char::is_whitespace)));
    assert!(Rule::all().all(|r| r.guidelines().iter().all(|g| g.starts_with("https://"))));
}

#[test]
fn test_misspelled_prefixed_aria_attribute() {
    let diags = lint_source(r#"fn c() { view! { <div attr:aria-labl="x"></div> } }"#);
    let d = diags
        .iter()
        .find(|d| d.rule == Rule::AriaProps)
        .expect("aria-props");
    assert!(d.message.contains("`attr:aria-labl`"), "{}", d.message);
}
