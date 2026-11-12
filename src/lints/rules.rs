//! Implementations of the individual lint rules.
//!
//! Each rule is a function that inspects one element and reports zero or more
//! diagnostics through the [`Emitter`]. Rules that report a *missing*
//! attribute use [`HtmlElement::may_have_attr`] so that elements with spread
//! attributes (`{..attrs}`) are not flagged.

use super::values::{is_valid_autocomplete, is_valid_lang};
use super::{Emitter, Rule};
use crate::dom::{Aria, AttributeName as A, MODULE_ROLES, Role, Tag};
use crate::parser::{HtmlAttribute, HtmlElement};

const LABEL_ATTRS: [A; 2] = [A::Aria(Aria::Label), A::Aria(Aria::LabelledBy)];
const NAME_ATTRS: [A; 3] = [A::Aria(Aria::Label), A::Aria(Aria::LabelledBy), A::Title];

/// Dispatch to the implementation of `rule`.
pub(super) fn check(rule: Rule, el: &HtmlElement, cx: &mut Emitter<'_>) {
    match rule {
        Rule::AltText => alt_text(el, cx),
        Rule::AnchorAmbiguousText => anchor_ambiguous_text(el, cx),
        Rule::AnchorHasContent => anchor_has_content(el, cx),
        Rule::AnchorIsValid => anchor_is_valid(el, cx),
        Rule::AriaActivedescendantHasTabindex => aria_activedescendant_has_tabindex(el, cx),
        Rule::AriaProps => aria_props(el, cx),
        Rule::AriaProptypes => aria_proptypes(el, cx),
        Rule::AriaRole => aria_role(el, cx),
        Rule::AriaUnsupportedElements => aria_unsupported_elements(el, cx),
        Rule::AutocompleteValid => autocomplete_valid(el, cx),
        Rule::ClickEventsHaveKeyEvents => click_events_have_key_events(el, cx),
        Rule::ControlHasAssociatedLabel => control_has_associated_label(el, cx),
        Rule::HeadingHasContent => heading_has_content(el, cx),
        Rule::HtmlHasLang => html_has_lang(el, cx),
        Rule::IframeHasTitle => iframe_has_title(el, cx),
        Rule::ImgRedundantAlt => img_redundant_alt(el, cx),
        Rule::InteractiveSupportsFocus => interactive_supports_focus(el, cx),
        Rule::LabelHasAssociatedControl => label_has_associated_control(el, cx),
        Rule::Lang => lang(el, cx),
        Rule::MediaHasCaption => media_has_caption(el, cx),
        Rule::MouseEventsHaveKeyEvents => mouse_events_have_key_events(el, cx),
        Rule::NoAccessKey => no_access_key(el, cx),
        Rule::NoAriaHiddenOnFocusable => no_aria_hidden_on_focusable(el, cx),
        Rule::NoAutofocus => no_autofocus(el, cx),
        Rule::NoDistractingElements => no_distracting_elements(el, cx),
        Rule::NoInteractiveElementToNoninteractiveRole => {
            no_interactive_element_to_noninteractive_role(el, cx)
        }
        Rule::NoNoninteractiveElementInteractions => no_noninteractive_element_interactions(el, cx),
        Rule::NoNoninteractiveElementToInteractiveRole => {
            no_noninteractive_element_to_interactive_role(el, cx)
        }
        Rule::NoNoninteractiveTabindex => no_noninteractive_tabindex(el, cx),
        Rule::NoRedundantRoles => no_redundant_roles(el, cx),
        Rule::NoStaticElementInteractions => no_static_element_interactions(el, cx),
        Rule::PreferTagOverRole => prefer_tag_over_role(el, cx),
        Rule::RoleHasRequiredAriaProps => role_has_required_aria_props(el, cx),
        Rule::RoleSupportsAriaProps => role_supports_aria_props(el, cx),
        Rule::Scope => scope(el, cx),
        Rule::TabindexNoPositive => tabindex_no_positive(el, cx),
    }
}

/// All attributes named `name` that have a static value, with that value.
fn static_attrs<'a>(
    el: &'a HtmlElement,
    name: &'a A,
) -> impl Iterator<Item = (&'a HtmlAttribute, &'a str)> + 'a {
    el.attributes
        .iter()
        .filter(move |a| &a.name == name)
        .filter_map(|a| a.value.as_static().map(|v| (a, v)))
}

/// The explicit `role` attribute together with the role it resolves to.
fn role_attr(el: &HtmlElement) -> Option<(&HtmlAttribute, Role)> {
    let attr = el.attr(&A::Role)?;
    el.explicit_role().map(|role| (attr, role))
}

/// Whether `el` is hidden from assistive technology or explicitly
/// presentational, which exempts it from interaction rules.
fn is_hidden_or_presentational(el: &HtmlElement) -> bool {
    el.is_aria_hidden() || matches!(el.explicit_role(), Some(Role::Presentation | Role::None))
}

/// Lowercase, drop punctuation and collapse whitespace.
fn normalize_text(text: &str) -> String {
    text.chars()
        .filter(|c| !matches!(c, '.' | ',' | '?' | '¿' | '!' | '‽' | '¡' | ';' | ':'))
        .collect::<String>()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn alt_text(el: &HtmlElement, cx: &mut Emitter<'_>) {
    let has_alt = el.may_have_attr(&A::Alt);
    let has_label = el.may_have_any_attr(&LABEL_ATTRS);
    match el.tag {
        Tag::Img => {
            let presentational =
                matches!(el.explicit_role(), Some(Role::Presentation | Role::None));
            if !has_alt && !presentational {
                cx.on_element(
                    "<img> element is missing an `alt` attribute.",
                    "Add an `alt` attribute with descriptive text, or `alt=\"\"` for decorative images.",
                );
            }
        }
        Tag::Area if !has_alt && !has_label => cx.on_element(
            "<area> element is missing an `alt` attribute.",
            "Add an `alt` attribute or `aria-label` / `aria-labelledby`.",
        ),
        Tag::Input if el.input_type().as_deref() == Some("image") && !has_alt && !has_label => cx
            .on_element(
                "<input type=\"image\"> is missing an `alt` attribute.",
                "Add an `alt` attribute or `aria-label` / `aria-labelledby`.",
            ),
        Tag::Object if !el.may_have_any_attr(&NAME_ATTRS) && !el.has_children => cx.on_element(
            "<object> element is missing alternative text.",
            "Add a `title` attribute, `aria-label` / `aria-labelledby`, or text content.",
        ),
        _ => {}
    }
}

fn anchor_ambiguous_text(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if el.tag != Tag::A {
        return;
    }
    const AMBIGUOUS: &[&str] = &["click here", "here", "link", "a link", "learn more"];
    // The accessible name: `aria-labelledby`, then `aria-label`, then the
    // content. `aria-labelledby` (which spread attributes may supply) and
    // dynamic content cannot be resolved statically.
    if el.may_have_attr(&A::Aria(Aria::LabelledBy)) {
        return;
    }
    let label = el.attr(&A::Aria(Aria::Label));
    let (name, attr) = match label.map(|a| (a, a.value.as_static())) {
        Some((_, None)) => return,
        // An empty `aria-label` is ignored.
        Some((attr, Some(label))) if !label.trim().is_empty() => (label.to_string(), Some(attr)),
        _ if el.has_dynamic_content => return,
        _ => (el.text.clone(), None),
    };
    if !AMBIGUOUS.contains(&normalize_text(&name).as_str()) {
        return;
    }
    let message = format!(
        "<a> element has ambiguous link text \"{}\". Link text should describe the link's purpose.",
        name.trim()
    );
    let help = "Use text that describes where the link goes or what it does, e.g. \"Read the installation guide\".";
    match attr {
        Some(attr) => cx.on_attr(attr, message, help),
        None => cx.on_element(message, help),
    }
}

fn anchor_has_content(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if el.tag == Tag::A
        && !el.has_children
        && !el.may_have_any_attr(&NAME_ATTRS)
        && !el.is_aria_hidden()
    {
        cx.on_element(
            "<a> element is missing content. Links must have discernible text.",
            "Add text content or an `aria-label` attribute.",
        );
    }
}

fn anchor_is_valid(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if el.tag != Tag::A {
        return;
    }
    for (attr, val) in static_attrs(el, &A::Href) {
        let href = val.trim().to_ascii_lowercase();
        if href.is_empty() || href == "#" || href.starts_with("javascript:") {
            cx.on_attr(
                attr,
                format!(
                    "<a> element has an invalid `href` value \"{val}\". Use a real URL, or a <button> for actions."
                ),
                "Use a meaningful `href`, or use a <button> element instead.",
            );
        }
    }
    if !el.may_have_attr(&A::Href) && el.has_attr(&A::OnClick) {
        cx.on_element(
            "<a> element without an `href` is used as a button.",
            "Use a <button> for actions, or give the link an `href` it navigates to.",
        );
    }
}

fn aria_activedescendant_has_tabindex(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if !el.is_natively_interactive()
        && el.has_attr(&A::Aria(Aria::ActiveDescendant))
        && !el.may_have_attr(&A::TabIndex)
    {
        cx.on_element(
            format!(
                "<{}> with `aria-activedescendant` must also have a `tabindex` attribute to be focusable.",
                el.tag
            ),
            "Add `tabindex=\"0\"` to make the element focusable.",
        );
    }
}

fn aria_props(el: &HtmlElement, cx: &mut Emitter<'_>) {
    for attr in &el.attributes {
        // Check the name without a Leptos `attr:` prefix, and Dioxus
        // `aria_*` spellings, but report it as written.
        if let A::Unknown(name) = &attr.name
            && let base = name.strip_prefix("attr:").unwrap_or(name)
            && (base.starts_with("aria-") || base.starts_with("aria_"))
        {
            cx.on_attr(
                attr,
                format!("Invalid ARIA attribute `{name}` on <{}>.", el.tag),
                "Check the spelling. See https://www.w3.org/TR/wai-aria-1.2/#state_prop_def for all valid attributes.",
            );
        }
    }
}

fn aria_proptypes(el: &HtmlElement, cx: &mut Emitter<'_>) {
    for attr in &el.attributes {
        let A::Aria(aria) = &attr.name else { continue };
        let vtype = aria.value_type();
        if let Some(val) = attr.value.as_static()
            && !vtype.is_valid(val.trim())
        {
            cx.on_attr(
                attr,
                format!(
                    "Invalid value \"{val}\" for `{aria}` on <{}>. Expected {}.",
                    el.tag,
                    vtype.expected_description()
                ),
                format!("Use {} for `{aria}`.", vtype.expected_description()),
            );
        }
    }
}

fn aria_role(el: &HtmlElement, cx: &mut Emitter<'_>) {
    for (attr, val) in static_attrs(el, &A::Role) {
        // A role attribute may list fallback roles separated by whitespace.
        for token in val.split_whitespace() {
            match token.parse::<Role>() {
                Ok(role) if role.is_abstract() => cx.on_attr(
                    attr,
                    format!(
                        "Abstract ARIA role \"{token}\" must not be used on <{}>. Abstract roles are for ontology purposes only.",
                        el.tag
                    ),
                    "Use a non-abstract role instead. See https://www.w3.org/TR/wai-aria-1.2/#abstract_roles",
                ),
                Ok(_) => {}
                Err(_) if MODULE_ROLES.contains(&token.to_ascii_lowercase().as_str()) => {}
                Err(_) => cx.on_attr(
                    attr,
                    format!("Invalid ARIA role \"{token}\" on <{}>.", el.tag),
                    "See https://www.w3.org/TR/wai-aria-1.2/#role_definitions for valid roles.",
                ),
            }
        }
    }
}

fn aria_unsupported_elements(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if el.tag.supports_aria() {
        return;
    }
    for attr in &el.attributes {
        if matches!(attr.name, A::Aria(_) | A::Role) {
            cx.on_attr(
                attr,
                format!(
                    "ARIA attribute `{}` is not supported on <{}>.",
                    attr.name, el.tag
                ),
                format!(
                    "Remove `{}`: the <{}> element does not support ARIA roles or properties.",
                    attr.name, el.tag
                ),
            );
        }
    }
}

fn autocomplete_valid(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if !matches!(el.tag, Tag::Input | Tag::Select | Tag::Textarea | Tag::Form) {
        return;
    }
    // A <form> only takes "on" or "off"; its controls take autofill tokens.
    let form = el.tag == Tag::Form;
    for (attr, val) in static_attrs(el, &A::Autocomplete) {
        let valid = if form {
            matches!(val.trim().to_ascii_lowercase().as_str(), "on" | "off")
        } else {
            is_valid_autocomplete(val)
        };
        if !valid {
            cx.on_attr(
                attr,
                format!("Invalid `autocomplete` value \"{val}\" on <{}>.", el.tag),
                if form {
                    "Use \"on\" or \"off\". Autofill tokens such as \"email\" belong on the form's controls."
                } else {
                    "Use a valid autocomplete token such as \"name\", \"email\", \"work tel\", \"shipping postal-code\", \"current-password\" or \"off\"."
                },
            );
        }
    }
}

fn click_events_have_key_events(el: &HtmlElement, cx: &mut Emitter<'_>) {
    // Interactive elements inherently handle keyboard events. An interactive
    // *role* (e.g. role="button") does not add keyboard behaviour, so it does
    // not exempt the element.
    if el.is_natively_interactive() || is_hidden_or_presentational(el) || !el.has_attr(&A::OnClick)
    {
        return;
    }
    if !el.has_any_attr(&[A::OnKeyDown, A::OnKeyUp, A::OnKeyPress]) {
        cx.on_element(
            format!(
                "<{}> with click handler must also have a keyboard event handler (onkeydown, onkeyup, or onkeypress) for accessibility.",
                el.tag
            ),
            "Add an `onkeydown` or `onkeyup` handler, or use an interactive element like <button> instead.",
        );
    }
}

fn control_has_associated_label(el: &HtmlElement, cx: &mut Emitter<'_>) {
    match el.tag {
        Tag::Input => {
            let Some(kind) = el.input_type() else { return };
            match kind.as_str() {
                // Not rendered, or labelled by the browser's default text.
                "hidden" | "submit" | "reset" => return,
                // Covered by the alt-text rule.
                "image" => return,
                "button" if el.may_have_attr(&A::Value) => return,
                _ => {}
            }
        }
        Tag::Button => {
            if el.has_children {
                return;
            }
        }
        Tag::Select | Tag::Textarea | Tag::Meter | Tag::Output | Tag::Progress => {}
        _ => return,
    }
    let labelled = el.may_have_any_attr(&NAME_ATTRS)
        || cx.doc.is_labelled_by_ancestor(el)
        || cx.doc.is_label_target(el);
    if !labelled && !el.is_aria_hidden() {
        cx.on_element(
            format!(
                "<{}> element has no associated label. Form controls must have a text label.",
                el.tag
            ),
            "Wrap it in a <label>, point a <label for=\"…\"> at its `id`, or add an `aria-label` / `aria-labelledby`.",
        );
    }
}

fn heading_has_content(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if el.tag.is_heading() && !el.has_children && !el.may_have_any_attr(&LABEL_ATTRS) {
        cx.on_element(
            format!(
                "<{}> element appears to be empty. Headings must have text content for accessibility.",
                el.tag
            ),
            "Add text content or an `aria-label` attribute.",
        );
    }
}

fn html_has_lang(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if el.tag == Tag::Html && !el.may_have_attr(&A::Lang) {
        cx.on_element(
            "<html> element is missing a `lang` attribute.",
            "Add a `lang` attribute (e.g., `lang=\"en\"`) to help screen readers determine the correct pronunciation.",
        );
    }
}

fn iframe_has_title(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if el.tag == Tag::Iframe && !el.may_have_any_attr(&NAME_ATTRS) && !el.is_aria_hidden() {
        cx.on_element(
            "<iframe> element is missing a `title` attribute.",
            "Add a `title` attribute that describes the iframe content.",
        );
    }
}

fn img_redundant_alt(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if el.tag != Tag::Img || el.is_aria_hidden() {
        return;
    }
    const REDUNDANT: &[&str] = &["image", "picture", "photo"];
    for (attr, val) in static_attrs(el, &A::Alt) {
        let lower = val.to_lowercase();
        let word = lower
            .split(|c: char| !c.is_alphanumeric())
            .find(|w| REDUNDANT.contains(w));
        if let Some(word) = word {
            cx.on_attr(
                attr,
                format!(
                    "<img> alt text contains the redundant word \"{word}\". Screen readers already announce images as images."
                ),
                "Describe what the image shows instead of stating it's an image.",
            );
        }
    }
}

fn interactive_supports_focus(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if el.is_natively_interactive() || el.is_aria_hidden() {
        return;
    }
    let interactive_role = el.explicit_role().is_some_and(|r| r.is_interactive());
    if interactive_role && el.has_event_handler() && !el.is_tabbable() {
        cx.on_element(
            format!(
                "<{}> with an interactive role must be focusable. Add a `tabindex` attribute.",
                el.tag
            ),
            "Add `tabindex=\"0\"` to make the element focusable, or use a natively interactive element like <button>.",
        );
    }
}

fn label_has_associated_control(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if el.tag != Tag::Label {
        return;
    }
    let has_control = el.may_have_attr(&A::For)
        || el.has_dynamic_content
        || cx.doc.descendants(el).iter().any(|d| d.is_labelable());
    let has_text =
        !el.text.is_empty() || el.has_dynamic_content || el.may_have_any_attr(&LABEL_ATTRS);

    if !has_control {
        cx.on_element(
            "<label> element has no associated form control.",
            "Add a `for` attribute matching a form control's `id`, or nest the control inside the label.",
        );
    } else if !has_text {
        cx.on_element(
            "<label> element has no text.",
            "Add text that describes the associated form control.",
        );
    }
}

fn lang(el: &HtmlElement, cx: &mut Emitter<'_>) {
    for (attr, val) in static_attrs(el, &A::Lang) {
        if !is_valid_lang(val) {
            cx.on_attr(
                attr,
                format!("The `lang` attribute value \"{val}\" is not a valid BCP 47 language tag."),
                "Use a valid BCP 47 language tag, e.g., \"en\", \"en-US\", \"fr\", \"de\", \"zh-Hans\".",
            );
        }
    }
}

fn media_has_caption(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if !matches!(el.tag, Tag::Video | Tag::Audio) || el.is_aria_hidden() {
        return;
    }
    // Muted video has no audio to caption.
    if el.tag == Tag::Video && el.has_attr(&A::Muted) {
        return;
    }
    // Tracks rendered dynamically cannot be checked.
    if el.has_dynamic_content || el.has_spread_attributes {
        return;
    }
    let has_captions = cx.doc.children(el).iter().any(|child| {
        child.tag == Tag::Track
            && child.attr(&A::Kind).is_some_and(|k| {
                k.value
                    .as_static()
                    .is_none_or(|v| v.trim().eq_ignore_ascii_case("captions"))
            })
    });
    if !has_captions {
        cx.on_element(
            format!("<{}> element must have captions.", el.tag),
            "Add a <track kind=\"captions\" src=\"…\"> child element.",
        );
    }
}

fn mouse_events_have_key_events(el: &HtmlElement, cx: &mut Emitter<'_>) {
    for (mouse, key, mouse_name, key_name) in [
        (A::OnMouseOver, A::OnFocus, "mouseover", "onfocus"),
        (A::OnMouseOut, A::OnBlur, "mouseout", "onblur"),
    ] {
        if let Some(attr) = el.attr(&mouse)
            && !el.has_attr(&key)
        {
            cx.on_attr(
                attr,
                format!(
                    "<{}> has a {mouse_name} event handler but no {key_name} handler. This can cause accessibility issues for keyboard users.",
                    el.tag
                ),
                format!(
                    "Add an `{key_name}` handler that mirrors the behavior of the `on{mouse_name}` handler."
                ),
            );
        }
    }
}

fn no_access_key(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if let Some(attr) = el.attr(&A::AccessKey) {
        cx.on_attr(
            attr,
            format!(
                "Avoid using the `accesskey` attribute on <{}>. Access keys create keyboard shortcuts that conflict with screen reader and keyboard commands.",
                el.tag
            ),
            "Remove the `accesskey` attribute.",
        );
    }
}

fn no_aria_hidden_on_focusable(el: &HtmlElement, cx: &mut Emitter<'_>) {
    // Like axe's aria-hidden-focus and eslint-plugin-jsx-a11y, this is about
    // the Tab order: an element taken out of it with `tabindex="-1"` passes.
    if !el.is_tabbable() {
        return;
    }
    if let Some(attr) = el.attr(&A::Aria(Aria::Hidden))
        && attr.value.is("true")
    {
        cx.on_attr(
            attr,
            format!(
                "<{}> element is focusable but has `aria-hidden=\"true\"`, which hides it from assistive technologies.",
                el.tag
            ),
            "Remove `aria-hidden=\"true\"`, or take the element out of the tab order with `tabindex=\"-1\"` (or `disabled`, for a form control).",
        );
    }
}

fn no_autofocus(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if let Some(attr) = el.attr(&A::AutoFocus)
        && !attr.value.is("false")
    {
        cx.on_attr(
            attr,
            format!(
                "Avoid using the `autofocus` attribute on <{}>. Autofocus can reduce usability and accessibility for sighted and non-sighted users.",
                el.tag
            ),
            "Remove the `autofocus` attribute.",
        );
    }
}

fn no_distracting_elements(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if matches!(el.tag, Tag::Marquee | Tag::Blink) {
        cx.on_element(
            format!(
                "<{}> elements are distracting and should not be used. They can cause accessibility issues for users with visual or cognitive disabilities.",
                el.tag
            ),
            "Use CSS animations or transitions instead.",
        );
    }
}

fn no_interactive_element_to_noninteractive_role(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if !el.is_natively_interactive() {
        return;
    }
    if let Some((attr, role)) = role_attr(el)
        && !role.is_interactive()
    {
        cx.on_attr(
            attr,
            format!(
                "Interactive element <{}> should not be assigned the non-interactive role \"{role}\".",
                el.tag
            ),
            "Remove the `role` attribute or use an appropriate interactive role.",
        );
    }
}

fn no_noninteractive_element_interactions(el: &HtmlElement, cx: &mut Emitter<'_>) {
    // Elements without an implicit role are handled by
    // no-static-element-interactions instead.
    if el.is_natively_interactive() || is_hidden_or_presentational(el) {
        return;
    }
    let Some(implicit) = el.implicit_role() else {
        return;
    };
    if implicit.is_interactive() || el.explicit_role().is_some_and(|r| r.is_interactive()) {
        return;
    }
    let has_handler = el.has_any_attr(&[A::OnClick, A::OnKeyDown, A::OnKeyUp, A::OnKeyPress]);
    if has_handler {
        cx.on_element(
            format!(
                "Non-interactive element <{}> should not have event handlers.",
                el.tag
            ),
            "Use an interactive element like <button> or <a>, or add an appropriate `role` attribute.",
        );
    }
}

/// Interactive roles that are conventionally allowed on non-interactive
/// semantic elements (the defaults of eslint-plugin-jsx-a11y).
fn allowed_interactive_role(tag: Tag, role: Role) -> bool {
    match tag {
        Tag::Ul | Tag::Ol => matches!(
            role,
            Role::ListBox
                | Role::Menu
                | Role::Menubar
                | Role::RadioGroup
                | Role::TabList
                | Role::Tree
                | Role::TreeGrid
        ),
        Tag::Li => matches!(
            role,
            Role::MenuItem
                | Role::MenuItemRadio
                | Role::MenuItemCheckbox
                | Role::Option
                | Role::Row
                | Role::Tab
                | Role::TreeItem
        ),
        Tag::Table => role == Role::Grid,
        Tag::Td => role == Role::GridCell,
        Tag::Fieldset => role == Role::RadioGroup,
        _ => false,
    }
}

fn no_noninteractive_element_to_interactive_role(el: &HtmlElement, cx: &mut Emitter<'_>) {
    // Only semantic, non-interactive elements: static elements like <div>
    // and <span> may take an interactive role (see interactive-supports-focus
    // and click-events-have-key-events for what they then need).
    let Some(implicit) = el.implicit_role() else {
        return;
    };
    if el.is_natively_interactive() || implicit.is_interactive() {
        return;
    }
    if let Some((attr, role)) = role_attr(el)
        && role.is_interactive()
        && !allowed_interactive_role(el.tag, role)
    {
        cx.on_attr(
            attr,
            format!(
                "Non-interactive element <{}> should not be assigned the interactive role \"{role}\".",
                el.tag
            ),
            "Use an element with the right semantics instead, e.g. <button>, <a> or <input>, or a <div>/<span> with the role.",
        );
    }
}

fn no_noninteractive_tabindex(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if el
        .role()
        .is_some_and(|r| r.is_interactive() || r == Role::TabPanel)
    {
        return;
    }
    if el.is_natively_interactive() {
        return;
    }
    for (attr, val) in static_attrs(el, &A::TabIndex) {
        if let Ok(index) = val.trim().parse::<i32>()
            && index >= 0
        {
            cx.on_attr(
                attr,
                format!(
                    "Non-interactive element <{}> should not have `tabindex=\"{index}\"`. Non-interactive elements should not be focusable.",
                    el.tag
                ),
                "Remove the `tabindex` attribute, or add an interactive role.",
            );
        }
    }
}

fn no_redundant_roles(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if let Some(implicit) = el.implicit_role()
        && let Some((attr, role)) = role_attr(el)
        && role == implicit
    {
        cx.on_attr(
            attr,
            format!(
                "Redundant role \"{role}\" on <{}>. This is the element's implicit role.",
                el.tag
            ),
            "Remove the `role` attribute.",
        );
    }
}

fn no_static_element_interactions(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if el.is_natively_interactive()
        || el.implicit_role().is_some()
        || el.may_have_attr(&A::Role)
        || el.is_aria_hidden()
    {
        return;
    }
    if el.has_event_handler() {
        cx.on_element(
            format!(
                "<{}> with event handler(s) must have a `role` attribute.",
                el.tag
            ),
            "Add a `role` attribute that describes the element's purpose, or use a semantic element like <button> or <a>.",
        );
    }
}

fn prefer_tag_over_role(el: &HtmlElement, cx: &mut Emitter<'_>) {
    let Some((attr, role)) = role_attr(el) else {
        return;
    };
    // Don't flag if the element already *is* the preferred tag.
    if el.tag.implicit_role() == Some(role) || el.implicit_role() == Some(role) {
        return;
    }
    if let Some(preferred) = role.preferred_tag() {
        cx.on_attr(
            attr,
            format!("Prefer using the {preferred} element instead of `role=\"{role}\"`."),
            format!(
                "Use {preferred} which has built-in semantics and keyboard behavior instead of relying on ARIA."
            ),
        );
    }
}

fn role_has_required_aria_props(el: &HtmlElement, cx: &mut Emitter<'_>) {
    let Some((attr, role)) = role_attr(el) else {
        return;
    };
    // Native semantics already supply the state: the level of <h1>-<h6>,
    // or the checked state of a checkbox/radio input given a toggle role.
    let checkable_input =
        el.tag == Tag::Input && matches!(el.input_type().as_deref(), Some("checkbox" | "radio"));
    let toggle_role = matches!(
        role,
        Role::Checkbox | Role::Radio | Role::Switch | Role::MenuItemCheckbox | Role::MenuItemRadio
    );
    let native = el.implicit_role() == Some(role)
        || (role == Role::Heading && el.tag.is_heading())
        || (checkable_input && toggle_role);

    let mut required: Vec<Aria> = if native {
        Vec::new()
    } else {
        role.required_aria_props().to_vec()
    };
    // A focusable separator is a widget that needs a value, which no element
    // (not even <hr>) supplies natively.
    if role == Role::Separator && el.is_focusable() && !required.contains(&Aria::ValueNow) {
        required.push(Aria::ValueNow);
    }
    let missing: Vec<String> = required
        .iter()
        .filter(|req| !el.may_have_attr(&A::Aria(**req)))
        .map(|a| format!("`{a}`"))
        .collect();
    if !missing.is_empty() {
        cx.on_attr(
            attr,
            format!(
                "<{}> with role=\"{role}\" is missing required ARIA properties: {}.",
                el.tag,
                missing.join(", ")
            ),
            format!("Add the required ARIA properties for the \"{role}\" role."),
        );
    }
}

fn role_supports_aria_props(el: &HtmlElement, cx: &mut Emitter<'_>) {
    let Some(role) = el.role() else { return };
    for attr in &el.attributes {
        if let A::Aria(aria) = &attr.name
            && !aria.is_supported_by_role(role)
        {
            cx.on_attr(
                attr,
                format!(
                    "The `{aria}` property is not supported by the \"{role}\" role on <{}>.",
                    el.tag
                ),
                format!(
                    "Remove the `{aria}` property, or change the role to one that supports it."
                ),
            );
        }
    }
}

fn scope(el: &HtmlElement, cx: &mut Emitter<'_>) {
    if el.tag == Tag::Th {
        return;
    }
    if let Some(attr) = el.attr(&A::Scope) {
        cx.on_attr(
            attr,
            format!(
                "The `scope` attribute should only be used on <th> elements, not <{}>.",
                el.tag
            ),
            "Remove the `scope` attribute, or use a <th> element.",
        );
    }
}

fn tabindex_no_positive(el: &HtmlElement, cx: &mut Emitter<'_>) {
    for (attr, val) in static_attrs(el, &A::TabIndex) {
        if let Ok(index) = val.trim().parse::<i32>()
            && index > 0
        {
            cx.on_attr(
                attr,
                format!(
                    "Avoid using positive `tabindex` value ({index}) on <{}>. This creates an unexpected tab order.",
                    el.tag
                ),
                "Use `tabindex=\"0\"` for focusable elements or `tabindex=\"-1\"` for programmatically focusable elements.",
            );
        }
    }
}
