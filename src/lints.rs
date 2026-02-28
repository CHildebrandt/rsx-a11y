//! Lint rules for ARIA and accessibility checks.
//!
//! Each lint checks a specific accessibility concern on parsed HTML elements
//! found within Yew/Leptos/Dioxus macro invocations.

use crate::dom::{Aria, AttributeName, Role, Tag};
use crate::parser::{AttrValue, HtmlElement};
use strum::{EnumIter, IntoEnumIterator, VariantArray};

/// Severity level for a lint diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// A violation that must be fixed (e.g. missing `alt` on `<img>`).
    Error,
    /// A likely problem that should be reviewed.
    Warning,
    /// A suggestion for improved accessibility.
    Info,
}

/// Accessibility lint rule identifiers.
///
/// Each variant corresponds to a single lint check. Rules are serialized in
/// `kebab-case` (e.g. `alt-text`, `aria-role`) for CLI flags and JSON output.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, EnumIter, VariantArray,
)]
#[serde(rename_all = "kebab-case")]
pub enum Rule {
    AltText,
    AnchorAmbiguousText,
    AnchorHasContent,
    AnchorIsValid,
    AriaActivedescendantHasTabindex,
    AriaProps,
    AriaProptypes,
    AriaRole,
    AriaUnsupportedElements,
    AutocompleteValid,
    ClickEventsHaveKeyEvents,
    ControlHasAssociatedLabel,
    HeadingHasContent,
    HtmlHasLang,
    IframeHasTitle,
    ImgRedundantAlt,
    InteractiveSupportsFocus,
    LabelHasAssociatedControl,
    Lang,
    MediaHasCaption,
    MouseEventsHaveKeyEvents,
    NoAccessKey,
    NoAriaHiddenOnFocusable,
    NoAutofocus,
    NoDistractingElements,
    NoInteractiveElementToNoninteractiveRole,
    NoNoninteractiveElementInteractions,
    NoNoninteractiveElementToInteractiveRole,
    NoNoninteractiveTabindex,
    NoRedundantRoles,
    NoStaticElementInteractions,
    PreferTagOverRole,
    RoleHasRequiredAriaProps,
    RoleSupportsAriaProps,
    Scope,
    TabindexNoPositive,
}

impl Rule {
    pub fn from_str(s: &str) -> Option<Rule> {
        serde_json::from_str(&format!("\"{}\"", s)).ok()
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(self)
            .map(|s| s.trim_matches('"').to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    }

    pub const fn description(&self) -> &'static str {
        match self {
            Rule::AltText => {
                "Enforce all elements that require alternative text have meaningful information to relay back to end user."
            }
            Rule::AnchorAmbiguousText => {
                "Enforce <a> text to not exactly match \"click here\", \"here\", \"link\", or \"a link\"."
            }
            Rule::AnchorHasContent => "Enforce all anchors to contain accessible content.",
            Rule::AnchorIsValid => "Enforce all anchors are valid, navigable elements.",
            Rule::AriaActivedescendantHasTabindex => {
                "Enforce elements with aria-activedescendant are tabbable."
            }
            Rule::AriaProps => "Enforce all aria-* props are valid.",
            Rule::AriaProptypes => "Enforce ARIA state and property values are valid.",
            Rule::AriaRole => {
                "Enforce that elements with ARIA roles must use a valid, non-abstract ARIA role."
            }
            Rule::AriaUnsupportedElements => {
                "Enforce that elements that do not support ARIA roles, states, and properties do not have those attributes."
            }
            Rule::AutocompleteValid => " 	Enforce that autocomplete attributes are used correctly.",
            Rule::ClickEventsHaveKeyEvents => {
                "Enforce a clickable non-interactive element has at least one keyboard event listener."
            }
            Rule::ControlHasAssociatedLabel => {
                "Enforce that a control (an interactive element) has a text label."
            }
            Rule::HeadingHasContent => {
                "Enforce heading (h1, h2, etc) elements contain accessible content."
            }
            Rule::HtmlHasLang => "Enforce <html> element has lang prop.",
            Rule::IframeHasTitle => "Enforce iframe elements have a title attribute.",
            Rule::ImgRedundantAlt => {
                "Enforce <img> alt prop does not contain the word \"image\", \"picture\", or \"photo\"."
            }
            Rule::InteractiveSupportsFocus => {
                "Enforce that elements with interactive handlers like onClick must be focusable."
            }
            Rule::LabelHasAssociatedControl => {
                "Enforce that a label tag has a text label and an associated control."
            }
            Rule::Lang => "Enforce lang attribute has a valid value.",
            Rule::MediaHasCaption => {
                "Enforces that <audio> and <video> elements must have a <track> for captions."
            }
            Rule::MouseEventsHaveKeyEvents => {
                "Enforce that onMouseOver/onMouseOut are accompanied by onFocus/onBlur for keyboard-only users."
            }
            Rule::NoAccessKey => {
                "Enforce that the accessKey prop is not used on any element to avoid complications with keyboard commands used by a screen reader."
            }
            Rule::NoAriaHiddenOnFocusable => {
                "Disallow aria-hidden=\"true\" from being set on focusable elements."
            }
            Rule::NoAutofocus => "Enforce autoFocus prop is not used.",
            Rule::NoDistractingElements => "Enforce distracting elements are not used.",
            Rule::NoInteractiveElementToNoninteractiveRole => {
                "Interactive elements should not be assigned non-interactive roles."
            }
            Rule::NoNoninteractiveElementInteractions => {
                "Non-interactive elements should not be assigned mouse or keyboard event listeners."
            }
            Rule::NoNoninteractiveElementToInteractiveRole => {
                "Non-interactive elements should not be assigned interactive roles."
            }
            Rule::NoNoninteractiveTabindex => {
                "Enforce tabIndex should only be declared on interactive elements."
            }
            Rule::NoRedundantRoles => {
                "Enforce explicit role property is not the same as implicit/default role property on element."
            }
            Rule::NoStaticElementInteractions => {
                "Enforce that non-interactive, visible elements (such as <div>) that have click handlers use the role attribute."
            }
            Rule::PreferTagOverRole => {
                "Enforces using semantic DOM elements over the ARIA role property."
            }
            Rule::RoleHasRequiredAriaProps => {
                "Enforce that elements with ARIA roles must have all required attributes for that role."
            }
            Rule::RoleSupportsAriaProps => {
                "Enforce that elements with explicit or implicit roles defined contain only aria-* properties supported by that role."
            }
            Rule::Scope => "Enforce scope prop is only used on <th> elements.",
            Rule::TabindexNoPositive => "Enforce tabIndex value is not greater than zero.",
        }
    }

    pub const fn guidelines(&self) -> &'static [&'static str] {
        match self {
            Rule::AltText => &["https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html"],
            Rule::AnchorAmbiguousText => &[],
            Rule::AnchorHasContent => &[
                "https://www.w3.org/WAI/WCAG21/Understanding/link-purpose-in-context",
                "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value",
            ],
            Rule::AnchorIsValid => &["https://www.w3.org/WAI/WCAG21/Understanding/keyboard"],
            Rule::AriaActivedescendantHasTabindex => &[""],
            Rule::AriaProps => &["https://www.w3.org/WAI/WCAG21/Understanding/name-role-value"],
            Rule::AriaProptypes => &["https://www.w3.org/WAI/WCAG21/Understanding/name-role-value"],
            Rule::AriaRole => &["https://www.w3.org/WAI/WCAG21/Understanding/name-role-value"],
            Rule::AriaUnsupportedElements => {
                &["https://www.w3.org/WAI/WCAG21/Understanding/name-role-value"]
            }
            Rule::AutocompleteValid => {
                &["https://www.w3.org/WAI/WCAG21/Understanding/identify-input-purpose"]
            }
            Rule::ClickEventsHaveKeyEvents => {
                &["https://www.w3.org/WAI/WCAG21/Understanding/keyboard"]
            }
            Rule::ControlHasAssociatedLabel => &[
                "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships",
                "https://www.w3.org/WAI/WCAG21/Understanding/labels-or-instructions",
                "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value",
            ],
            Rule::HeadingHasContent => &[
                "https://www.w3.org/TR/UNDERSTANDING-WCAG20/navigation-mechanisms-descriptive.html",
            ],
            Rule::HtmlHasLang => &["https://www.w3.org/WAI/WCAG21/Understanding/language-of-page"],
            Rule::IframeHasTitle => &[
                "https://www.w3.org/WAI/WCAG21/Understanding/bypass-blocks",
                "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value",
            ],
            Rule::ImgRedundantAlt => &[],
            Rule::InteractiveSupportsFocus => {
                &["https://www.w3.org/WAI/WCAG21/Understanding/keyboard"]
            }
            Rule::LabelHasAssociatedControl => &[
                "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships",
                "https://www.w3.org/WAI/WCAG21/Understanding/labels-or-instructions",
                "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value",
            ],
            Rule::Lang => &["https://www.w3.org/WAI/WCAG21/Understanding/language-of-page"],
            Rule::MediaHasCaption => &[
                "https://www.w3.org/WAI/WCAG21/Understanding/captions-prerecorded.html",
                "https://www.w3.org/WAI/WCAG21/Understanding/audio-description-or-media-alternative-prerecorded.html",
            ],
            Rule::MouseEventsHaveKeyEvents => {
                &["https://www.w3.org/WAI/WCAG21/Understanding/keyboard"]
            }
            Rule::NoAccessKey => &[],
            Rule::NoAriaHiddenOnFocusable => &[],
            Rule::NoAutofocus => &[],
            Rule::NoDistractingElements => {
                &["https://www.w3.org/WAI/WCAG21/Understanding/pause-stop-hide"]
            }
            Rule::NoInteractiveElementToNoninteractiveRole => {
                &["https://www.w3.org/WAI/WCAG21/Understanding/name-role-value"]
            }
            Rule::NoNoninteractiveElementInteractions => {
                &["https://www.w3.org/WAI/WCAG21/Understanding/name-role-value"]
            }
            Rule::NoNoninteractiveElementToInteractiveRole => {
                &["https://www.w3.org/WAI/WCAG21/Understanding/name-role-value"]
            }
            Rule::NoNoninteractiveTabindex => {
                &["https://www.w3.org/WAI/WCAG21/Understanding/keyboard"]
            }
            Rule::NoRedundantRoles => &[],
            Rule::NoStaticElementInteractions => {
                &["https://www.w3.org/WAI/WCAG21/Understanding/name-role-value"]
            }
            Rule::PreferTagOverRole => &["https://www.w3.org/TR/wai-aria-1.0/roles"],
            Rule::RoleHasRequiredAriaProps => {
                &["https://www.w3.org/WAI/WCAG21/Understanding/name-role-value"]
            }
            Rule::RoleSupportsAriaProps => {
                &["https://www.w3.org/WAI/WCAG21/Understanding/name-role-value"]
            }
            Rule::Scope => &[
                "https://www.w3.org/WAI/WCAG21/Understanding/info-and-relationships",
                "https://www.w3.org/WAI/WCAG21/Understanding/parsing",
            ],
            Rule::TabindexNoPositive => {
                &["https://www.w3.org/WAI/WCAG21/Understanding/focus-order"]
            }
        }
    }

    pub const fn resources(&self) -> &'static [&'static str] {
        match self {
            Rule::AltText => &[
                "https://dequeuniversity.com/rules/axe/3.2/object-alt",
                "https://dequeuniversity.com/rules/axe/3.2/image-alt",
                "https://dequeuniversity.com/rules/axe/3.2/input-image-alt",
                "https://dequeuniversity.com/rules/axe/3.2/area-alt",
            ],
            Rule::AnchorAmbiguousText => &[
                "https://webaim.org/techniques/hypertext/",
                "https://dequeuniversity.com/checklists/web/links",
            ],
            Rule::AnchorHasContent => &["https://dequeuniversity.com/rules/axe/3.2/link-name"],
            Rule::AnchorIsValid => &[
                "https://webaim.org/techniques/hypertext/",
                "https://marcysutton.com/links-vs-buttons-in-modern-web-applications/",
                "https://www.w3.org/TR/using-aria/#NOTES",
            ],
            Rule::AriaActivedescendantHasTabindex => &[
                "https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/ARIA_Techniques/Using_the_aria-activedescendant_attribute",
            ],
            Rule::AriaProps => &[],
            Rule::AriaProptypes => &[
                "https://www.w3.org/TR/wai-aria/#states_and_properties",
                "https://github.com/GoogleChrome/accessibility-developer-tools/wiki/Audit-Rules#ax_aria_04",
            ],
            Rule::AriaRole => &[
                "https://github.com/GoogleChrome/accessibility-developer-tools/wiki/Audit-Rules#ax_aria_01",
                "https://www.w3.org/TR/dpub-aria-1.0/",
                "https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/ARIA_Techniques",
            ],
            Rule::AriaUnsupportedElements => &[
                "https://github.com/GoogleChrome/accessibility-developer-tools/wiki/Audit-Rules#ax_aria_12",
                "https://www.w3.org/TR/dpub-aria-1.0/",
            ],
            Rule::AutocompleteValid => &[
                "https://dequeuniversity.com/rules/axe/3.2/autocomplete-valid",
                "https://www.w3.org/TR/html52/sec-forms.html#autofilling-form-controls-the-autocomplete-attribute",
            ],
            Rule::ClickEventsHaveKeyEvents => &[],
            Rule::ControlHasAssociatedLabel => &[],
            Rule::HeadingHasContent => &["https://dequeuniversity.com/rules/axe/3.2/empty-heading"],
            Rule::HtmlHasLang => &[
                "https://dequeuniversity.com/rules/axe/3.2/html-has-lang",
                "https://dequeuniversity.com/rules/axe/3.2/html-lang-valid",
            ],
            Rule::IframeHasTitle => &["https://dequeuniversity.com/rules/axe/3.2/frame-title"],
            Rule::ImgRedundantAlt => &["https://webaim.org/techniques/alttext/"],
            Rule::InteractiveSupportsFocus => &[
                "https://github.com/GoogleChrome/accessibility-developer-tools/wiki/Audit-Rules#ax_focus_02",
                "https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/ARIA_Techniques/Using_the_button_role#Keyboard_and_focus",
                "https://www.w3.org/TR/wai-aria-practices-1.1/#kbd_generalnav",
                "https://www.w3.org/TR/wai-aria-practices-1.1/#aria_ex",
            ],
            Rule::LabelHasAssociatedControl => &[],
            Rule::Lang => &[
                "https://dequeuniversity.com/rules/axe/3.2/valid-lang",
                "https://www.w3.org/International/articles/language-tags/",
                "https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry",
            ],
            Rule::MediaHasCaption => &[
                "https://dequeuniversity.com/rules/axe/2.1/audio-caption",
                "https://dequeuniversity.com/rules/axe/2.1/video-caption",
            ],
            Rule::MouseEventsHaveKeyEvents => &[],
            Rule::NoAccessKey => &["https://webaim.org/techniques/keyboard/accesskey#spec"],
            Rule::NoAriaHiddenOnFocusable => &[
                "https://dequeuniversity.com/rules/axe/html/4.4/aria-hidden-focus",
                "https://www.w3.org/WAI/standards-guidelines/act/rules/6cfa84/proposed/",
                "https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Attributes/aria-hidden",
            ],
            Rule::NoAutofocus => &[
                "https://html.spec.whatwg.org/multipage/interaction.html#attr-fe-autofocus",
                "https://www.brucelawson.co.uk/2009/the-accessibility-of-html-5-autofocus/",
            ],
            Rule::NoDistractingElements => &[
                "https://dequeuniversity.com/rules/axe/3.2/marquee",
                "https://dequeuniversity.com/rules/axe/3.2/blink",
            ],
            Rule::NoInteractiveElementToNoninteractiveRole => &[
                "https://www.w3.org/TR/wai-aria/#states_and_properties",
                "https://github.com/GoogleChrome/accessibility-developer-tools/wiki/Audit-Rules#ax_aria_04",
                "https://www.w3.org/TR/wai-aria-1.1/#usage_intro",
                "https://www.w3.org/TR/wai-aria-practices-1.1/#aria_ex",
                "https://www.w3.org/TR/wai-aria-practices-1.1/#kbd_generalnav",
                "https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/ARIA_Techniques/Using_the_button_role#Keyboard_and_focus",
            ],
            Rule::NoNoninteractiveElementInteractions => &[
                "https://www.w3.org/TR/wai-aria-1.1/#usage_intro",
                "https://www.w3.org/TR/wai-aria-practices-1.1/#aria_ex",
                "https://www.w3.org/TR/wai-aria-practices-1.1/#kbd_generalnav",
                "https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/ARIA_Techniques/Using_the_button_role#Keyboard_and_focus",
            ],
            Rule::NoNoninteractiveElementToInteractiveRole => &[
                "https://www.w3.org/TR/wai-aria-1.1/#usage_intro",
                "https://www.w3.org/TR/wai-aria-practices-1.1/#aria_ex",
                "https://www.w3.org/TR/wai-aria-practices-1.1/#kbd_generalnav",
                "https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/ARIA_Techniques/Using_the_button_role#Keyboard_and_focus",
            ],
            Rule::NoNoninteractiveTabindex => {
                &["https://www.w3.org/TR/wai-aria-practices-1.1/#kbd_generalnav"]
            }
            Rule::NoRedundantRoles => &[
                "https://www.w3.org/TR/using-aria/#aria-does-nothing",
                "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/img#identifying_svg_as_an_image",
            ],
            Rule::NoStaticElementInteractions => &[
                "https://www.w3.org/TR/wai-aria-1.1/#usage_intro",
                "https://www.w3.org/TR/wai-aria-practices-1.1/#aria_ex",
                "https://www.w3.org/TR/wai-aria-practices-1.1/#kbd_generalnav",
                "https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/ARIA_Techniques/Using_the_button_role#Keyboard_and_focus",
            ],
            Rule::PreferTagOverRole => {
                &["https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Roles"]
            }
            Rule::RoleHasRequiredAriaProps => &[
                "https://www.w3.org/TR/wai-aria/#roles",
                "https://github.com/GoogleChrome/accessibility-developer-tools/wiki/Audit-Rules#ax_aria_03",
            ],
            Rule::RoleSupportsAriaProps => &[
                "https://www.w3.org/TR/wai-aria/#states_and_properties",
                "https://github.com/GoogleChrome/accessibility-developer-tools/wiki/Audit-Rules#ax_aria_10",
            ],
            Rule::Scope => &["https://dequeuniversity.com/rules/axe/3.5/scope-attr-valid"],
            Rule::TabindexNoPositive => &[
                "https://github.com/GoogleChrome/accessibility-developer-tools/wiki/Audit-Rules#ax_focus_03",
            ],
        }
    }

    pub fn check(&self, element: &HtmlElement) -> Option<LintDiagnostic> {
        match self {
            Rule::AltText => {
                let has_alt = element
                    .attributes
                    .iter()
                    .any(|a| a.name == AttributeName::Alt);
                let has_role_presentation = element.attributes.iter().any(|a| {
                    a.name == AttributeName::Role
                    && matches!(&a.value, Some(AttrValue::Static(v)) if v == "presentation" || v == "none")
                });
                let has_aria_label = element.attributes.iter().any(|a| {
                    a.name == AttributeName::Aria(Aria::Label)
                        || a.name == AttributeName::Aria(Aria::LabelledBy)
                });

                match element.tag {
                    // <img> must have alt (or role="presentation"/"none")
                    Tag::Img => {
                        if !has_alt && !has_role_presentation {
                            return Some(LintDiagnostic {
                                rule: Rule::AltText,
                                message: "<img> element is missing an `alt` attribute.".to_string(),
                                severity: Severity::Error,
                                file: element.file.clone(),
                                line: element.line,
                                column: element.column,
                                element: element.tag.clone(),
                                help: Some(
                                    "Add an `alt` attribute with descriptive text, or `alt=\"\"` for decorative images, \
                                    or `role=\"presentation\"` / `role=\"none\"`."
                                        .to_string(),
                                ),
                            });
                        }
                    }
                    // <area> must have alt or aria-label/aria-labelledby
                    Tag::Area => {
                        if !has_alt && !has_aria_label {
                            return Some(LintDiagnostic {
                                rule: Rule::AltText,
                                message: "<area> element is missing an `alt` attribute."
                                    .to_string(),
                                severity: Severity::Error,
                                file: element.file.clone(),
                                line: element.line,
                                column: element.column,
                                element: element.tag.clone(),
                                help: Some(
                                    "Add an `alt` attribute or `aria-label` / `aria-labelledby`."
                                        .to_string(),
                                ),
                            });
                        }
                    }
                    // <input type="image"> must have alt or aria-label/aria-labelledby
                    Tag::Input => {
                        let is_image_input = element.attributes.iter().any(|a| {
                            a.name == AttributeName::Type
                                && matches!(&a.value, Some(AttrValue::Static(v)) if v == "image")
                        });
                        if is_image_input && !has_alt && !has_aria_label {
                            return Some(LintDiagnostic {
                                rule: Rule::AltText,
                                message: "<input type=\"image\"> is missing an `alt` attribute."
                                    .to_string(),
                                severity: Severity::Error,
                                file: element.file.clone(),
                                line: element.line,
                                column: element.column,
                                element: element.tag.clone(),
                                help: Some(
                                    "Add an `alt` attribute or `aria-label` / `aria-labelledby`."
                                        .to_string(),
                                ),
                            });
                        }
                    }
                    // <object> must have title, aria-label/aria-labelledby or children
                    Tag::Object => {
                        let has_title = element
                            .attributes
                            .iter()
                            .any(|a| a.name == AttributeName::Title);
                        if !has_title && !has_aria_label && !element.has_children {
                            return Some(LintDiagnostic {
                                rule: Rule::AltText,
                                message: "<object> element is missing alternative text.".to_string(),
                                severity: Severity::Error,
                                file: element.file.clone(),
                                line: element.line,
                                column: element.column,
                                element: element.tag.clone(),
                                help: Some(
                                    "Add a `title` attribute, `aria-label` / `aria-labelledby`, or text content.".to_string(),
                                ),
                            });
                        }
                    }
                    _ => {}
                }
            }
            Rule::AnchorAmbiguousText => {
                if element.tag != Tag::A {
                    return None;
                }

                const AMBIGUOUS_TEXTS: &[&str] =
                    &["click here", "here", "link", "a link", "learn more"];

                // Check aria-label and title for ambiguous text.
                for attr in &element.attributes {
                    let is_label = attr.name == AttributeName::Aria(Aria::Label)
                        || attr.name == AttributeName::Title;
                    if is_label {
                        if let Some(AttrValue::Static(ref val)) = attr.value {
                            let lower = val.to_lowercase();
                            let trimmed = lower.trim();
                            if AMBIGUOUS_TEXTS.contains(&trimmed) {
                                return Some(LintDiagnostic {
                                    rule: Rule::AnchorAmbiguousText,
                                    message: format!(
                                        "<a> element has ambiguous link text \"{}\". Link text should be descriptive of the link's purpose.",
                                        val
                                    ),
                                    severity: Severity::Warning,
                                    file: element.file.clone(),
                                    line: attr.line,
                                    column: attr.column,
                                    element: element.tag.clone(),
                                    help: Some(
                                        "Use text that describes the purpose of the link, such as where the link goes or what it does.".to_string()
                                    ),
                                });
                            }
                        }
                    }
                }
            }
            Rule::AnchorHasContent => {
                if element.tag != Tag::A {
                    return None;
                }

                let has_accessible_name = element.attributes.iter().any(|a| {
                    a.name == AttributeName::Aria(Aria::Label)
                        || a.name == AttributeName::Aria(Aria::LabelledBy)
                        || a.name == AttributeName::Title
                });

                if !element.has_children && !has_accessible_name {
                    return Some(LintDiagnostic {
                        rule: Rule::AnchorHasContent,
                        message:
                            "<a> element is missing content. Links must have discernible text."
                                .to_string(),
                        severity: Severity::Warning,
                        file: element.file.clone(),
                        line: element.line,
                        column: element.column,
                        element: element.tag.clone(),
                        help: Some("Add text content or an `aria-label` attribute.".to_string()),
                    });
                }
            }
            Rule::AnchorIsValid => {
                if element.tag != Tag::A {
                    return None;
                }
                for attr in &element.attributes {
                    if attr.name == AttributeName::Href {
                        if let Some(AttrValue::Static(ref val)) = attr.value {
                            if val == "#" || val.is_empty() || val == "javascript:void(0)" {
                                return Some(LintDiagnostic {
                                    rule: Rule::AnchorIsValid,
                                    message: format!(
                                        "<a> element has an invalid `href` value \"{}\". \
                                        Use a real URL or use a <button> for actions.",
                                        val
                                    ),
                                    severity: Severity::Warning,
                                    file: element.file.clone(),
                                    line: attr.line,
                                    column: attr.column,
                                    element: element.tag.clone(),
                                    help: Some(
                                        "Use a meaningful `href`, or use a <button> element instead."
                                            .to_string(),
                                    ),
                                });
                            }
                        }
                    }
                }
            }
            Rule::AriaActivedescendantHasTabindex => {
                if element.tag.is_interactive() {
                    return None;
                }
                let has_activedescendant = element
                    .attributes
                    .iter()
                    .any(|a| a.name == AttributeName::Aria(Aria::ActiveDescendant));

                if has_activedescendant {
                    let has_tabindex = element
                        .attributes
                        .iter()
                        .any(|a| a.name == AttributeName::TabIndex);

                    if !has_tabindex {
                        return Some(LintDiagnostic {
                            rule: Rule::AriaActivedescendantHasTabindex,
                            message: format!(
                                "<{}> with `aria-activedescendant` must also have a `tabindex` attribute to be focusable.",
                                element.tag
                            ),
                            severity: Severity::Warning,
                            file: element.file.clone(),
                            line: element.line,
                            column: element.column,
                            element: element.tag.clone(),
                            help: Some(
                                "Add `tabindex=\"0\"` to make the element focusable.".to_string(),
                            ),
                        });
                    }
                }
            }
            Rule::AriaProps => {
                for attr in &element.attributes {
                    if let AttributeName::Unknown(unknown_value) = &attr.name {
                        if unknown_value.starts_with("aria-") {
                            return Some(LintDiagnostic {
                                rule: Rule::AriaProps,
                                message: format!(
                                    "Invalid ARIA attribute `{}` on <{}>.",
                                    attr.name, element.tag
                                ),
                                severity: Severity::Error,
                                file: element.file.clone(),
                                line: attr.line,
                                column: attr.column,
                                element: element.tag.clone(),
                                help: Some(format!(
                                    "Did you mean one of: aria-label, aria-labelledby, aria-hidden, aria-describedby? See https://www.w3.org/TR/wai-aria-1.2/#state_prop_def for all valid attributes."
                                )),
                            });
                        }
                    }
                }
            }
            Rule::AriaProptypes => {
                for attr in &element.attributes {
                    match &attr.name {
                        AttributeName::Aria(aria) => {
                            let vtype = aria.value_type();
                            if let Some(AttrValue::Static(ref val)) = attr.value {
                                if !vtype.is_valid(val) {
                                    return Some(LintDiagnostic {
                                        rule: Rule::AriaProptypes,
                                        message: format!(
                                            "Invalid value \"{}\" for `{}` on <{}>. Expected {}.",
                                            val,
                                            attr.name,
                                            element.tag,
                                            vtype.expected_description()
                                        ),
                                        severity: Severity::Error,
                                        file: element.file.clone(),
                                        line: attr.line,
                                        column: attr.column,
                                        element: element.tag.clone(),
                                        help: None,
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Rule::AriaRole => {
                for attr in &element.attributes {
                    if attr.name == AttributeName::Role {
                        if let Some(AttrValue::Static(ref val)) = attr.value {
                            // Role can be a space-separated list of roles (fallback roles)
                            for role_str in val.split_whitespace() {
                                match Role::from_str(role_str) {
                                    Some(role) if role.is_abstract() => {
                                        return Some(LintDiagnostic {
                                            rule: Rule::AriaRole,
                                            message: format!(
                                                "Abstract ARIA role \"{}\" must not be used on <{}>. \
                                                Abstract roles are for ontology purposes only.",
                                                role_str, element.tag
                                            ),
                                            severity: Severity::Error,
                                            file: element.file.clone(),
                                            line: attr.line,
                                            column: attr.column,
                                            element: element.tag.clone(),
                                            help: Some(
                                                "Use a non-abstract role instead. See https://www.w3.org/TR/wai-aria-1.2/#abstract_roles"
                                                    .to_string(),
                                            ),
                                        });
                                    }
                                    Some(_) => { /* valid concrete role */ }
                                    // Unknown role string
                                    None => {
                                        return Some(LintDiagnostic {
                                            rule: Rule::AriaRole,
                                            message: format!(
                                                "Invalid ARIA role \"{}\" on <{}>.",
                                                role_str, element.tag
                                            ),
                                            severity: Severity::Error,
                                            file: element.file.clone(),
                                            line: attr.line,
                                            column: attr.column,
                                            element: element.tag.clone(),
                                            help: Some(format!(
                                                "See https://www.w3.org/TR/wai-aria-1.2/#role_definitions for valid roles."
                                            )),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Rule::AriaUnsupportedElements => {
                if !element.tag.supports_aria() {
                    for attr in &element.attributes {
                        match attr.name {
                            AttributeName::Aria(_) | AttributeName::Role => {
                                return Some(LintDiagnostic {
                                    rule: Rule::AriaUnsupportedElements,
                                    message: format!(
                                        "ARIA attribute `{}` is not supported on <{}>.",
                                        attr.name, element.tag
                                    ),
                                    severity: Severity::Error,
                                    file: element.file.clone(),
                                    line: attr.line,
                                    column: attr.column,
                                    element: element.tag.clone(),
                                    help: Some(format!(
                                        "The <{}> element does not support ARIA roles or properties.",
                                        element.tag
                                    )),
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
            Rule::AutocompleteValid => {
                // Only applies to form elements that accept autocomplete
                if !matches!(element.tag, Tag::Input | Tag::Select | Tag::Textarea) {
                    return None;
                }
                for attr in &element.attributes {
                    if attr.name == AttributeName::Autocomplete {
                        if let Some(AttrValue::Static(ref val)) = attr.value {
                            if !is_valid_autocomplete(val) {
                                return Some(LintDiagnostic {
                                    rule: Rule::AutocompleteValid,
                                    message: format!(
                                        "Invalid `autocomplete` value \"{}\" on <{}>.",
                                        val, element.tag
                                    ),
                                    severity: Severity::Error,
                                    file: element.file.clone(),
                                    line: attr.line,
                                    column: attr.column,
                                    element: element.tag.clone(),
                                    help: Some(
                                        "Use a valid autocomplete value such as \"name\", \"email\", \"username\", \"current-password\", \"street-address\", \"off\", etc."
                                            .to_string(),
                                    ),
                                });
                            }
                        }
                    }
                }
            }
            Rule::ClickEventsHaveKeyEvents => {
                // Interactive elements inherently handle keyboard events
                if element.tag.is_interactive() {
                    return None;
                }

                let has_click = element
                    .attributes
                    .iter()
                    .any(|a| a.name == AttributeName::OnClick);

                if !has_click {
                    return None;
                }

                let has_key_handler = element.attributes.iter().any(|a| {
                    a.name == AttributeName::OnKeyDown
                        || a.name == AttributeName::OnKeyUp
                        || a.name == AttributeName::OnKeyPress
                });

                // An interactive role (e.g. role="button") changes semantics for screen
                // readers but does NOT add keyboard behaviour. The element still needs an
                // explicit key handler.
                if !has_key_handler {
                    return Some(LintDiagnostic {
                        rule: Rule::ClickEventsHaveKeyEvents,
                        message: format!(
                            "<{}> with click handler must also have a keyboard event handler (onkeydown, onkeyup, or onkeypress) for accessibility.",
                            element.tag
                        ),
                        severity: Severity::Warning,
                        file: element.file.clone(),
                        line: element.line,
                        column: element.column,
                        element: element.tag.clone(),
                        help: Some(
                            "Add an `onkeydown` or `onkeyup` handler, or use an interactive element like <button> instead."
                                .to_string(),
                        ),
                    });
                }
            }
            Rule::ControlHasAssociatedLabel => {
                // Per jsx-a11y: interactive controls must have a text label.
                let is_control = matches!(
                    element.tag,
                    Tag::Button
                        | Tag::Input
                        | Tag::Select
                        | Tag::Textarea
                        | Tag::Meter
                        | Tag::Output
                        | Tag::Progress
                );
                if !is_control {
                    return None;
                }

                let has_label = element.attributes.iter().any(|a| {
                    a.name == AttributeName::Aria(Aria::Label)
                        || a.name == AttributeName::Aria(Aria::LabelledBy)
                        || a.name == AttributeName::Title
                });

                // Children can contain text labels
                if !has_label && !element.has_children {
                    return Some(LintDiagnostic {
                        rule: Rule::ControlHasAssociatedLabel,
                        message: format!(
                            "<{}> element has no associated label. Interactive controls must have a text label.",
                            element.tag
                        ),
                        severity: Severity::Warning,
                        file: element.file.clone(),
                        line: element.line,
                        column: element.column,
                        element: element.tag.clone(),
                        help: Some(
                            "Add an `aria-label`, `aria-labelledby`, or `title` attribute, or use a <label>.".to_string(),
                        ),
                    });
                }
            }
            Rule::HeadingHasContent => {
                if !element.tag.is_heading() {
                    return None;
                }

                let has_aria_label = element.attributes.iter().any(|a| {
                    a.name == AttributeName::Aria(Aria::Label)
                        || a.name == AttributeName::Aria(Aria::LabelledBy)
                });

                if !element.has_children && !has_aria_label {
                    return Some(LintDiagnostic {
                        rule: Rule::HeadingHasContent,
                        message: format!(
                            "<{}> element appears to be empty. Headings must have text content \
                            for accessibility.",
                            element.tag
                        ),
                        severity: Severity::Warning,
                        file: element.file.clone(),
                        line: element.line,
                        column: element.column,
                        element: element.tag.clone(),
                        help: Some("Add text content or an `aria-label` attribute.".to_string()),
                    });
                }
            }
            Rule::HtmlHasLang => {
                if element.tag != Tag::Html {
                    return None;
                }

                let has_lang = element
                    .attributes
                    .iter()
                    .any(|a| a.name == AttributeName::Lang);

                if !has_lang {
                    return Some(LintDiagnostic {
                        rule: Rule::HtmlHasLang,
                        message: "<html> element is missing a `lang` attribute.".to_string(),
                        severity: Severity::Warning,
                        file: element.file.clone(),
                        line: element.line,
                        column: element.column,
                        element: element.tag.clone(),
                        help: Some(
                            "Add a `lang` attribute (e.g., `lang=\"en\"`) to help screen readers determine the correct pronunciation."
                                .to_string(),
                        ),
                    });
                }
            }
            Rule::IframeHasTitle => {
                if element.tag != Tag::Iframe {
                    return None;
                }

                let has_title = element
                    .attributes
                    .iter()
                    .any(|a| a.name == AttributeName::Title);
                let has_aria = element.attributes.iter().any(|a| {
                    a.name == AttributeName::Aria(Aria::Label)
                        || a.name == AttributeName::Aria(Aria::LabelledBy)
                });
                let has_hidden = element.attributes.iter().any(|a| {
                    a.name == AttributeName::Aria(Aria::Hidden)
                        && matches!(&a.value, Some(AttrValue::Static(v)) if v == "true")
                });

                if !has_title && !has_aria && !has_hidden {
                    return Some(LintDiagnostic {
                        rule: Rule::IframeHasTitle,
                        message: "<iframe> element is missing a `title` attribute.".to_string(),
                        severity: Severity::Warning,
                        file: element.file.clone(),
                        line: element.line,
                        column: element.column,
                        element: element.tag.clone(),
                        help: Some(
                            "Add a `title` attribute that describes the iframe content."
                                .to_string(),
                        ),
                    });
                }
            }
            Rule::ImgRedundantAlt => {
                if element.tag != Tag::Img {
                    return None;
                }

                for attr in &element.attributes {
                    if attr.name == AttributeName::Alt {
                        if let Some(AttrValue::Static(ref val)) = attr.value {
                            let lower = val.to_lowercase();
                            let redundant_words = ["image", "picture", "photo", "icon", "graphic"];
                            for word in &redundant_words {
                                if lower.contains(word) {
                                    return Some(LintDiagnostic {
                                        rule: Rule::ImgRedundantAlt,
                                        message: format!(
                                            "<img> alt text contains the redundant word \"{}\". \
                                            Screen readers already announce images as images.",
                                            word
                                        ),
                                        severity: Severity::Warning,
                                        file: element.file.clone(),
                                        line: attr.line,
                                        column: attr.column,
                                        element: element.tag.clone(),
                                        help: Some(
                                            "Describe what the image shows instead of stating it's an image."
                                                .to_string(),
                                        ),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            Rule::InteractiveSupportsFocus => {
                // Skip natively interactive elements (already focusable)
                if element.tag.is_interactive() {
                    return None;
                }

                // Check if element has an interactive role
                let has_interactive_role = element.attributes.iter().any(|a| {
                    a.name == AttributeName::Role
                        && matches!(&a.value, Some(AttrValue::Static(v)) if Role::from_str(v).is_some_and(|r| r.is_interactive()))
                });
                if !has_interactive_role {
                    return None;
                }

                // Check if element has an event handler
                if !element.has_event_handler() {
                    return None;
                }

                // Check if element is focusable (has tabindex)
                if !element.is_focusable() {
                    return Some(LintDiagnostic {
                        rule: Rule::InteractiveSupportsFocus,
                        message: format!(
                            "<{}> with an interactive role must be focusable. Add a `tabindex` attribute.",
                            element.tag
                        ),
                        severity: Severity::Warning,
                        file: element.file.clone(),
                        line: element.line,
                        column: element.column,
                        element: element.tag.clone(),
                        help: Some(
                            "Add `tabindex=\"0\"` to make the element focusable, or use a natively interactive element like <button>."
                                .to_string(),
                        ),
                    });
                }
            }
            Rule::LabelHasAssociatedControl => {
                if element.tag != Tag::Label {
                    return None;
                }

                let has_for = element
                    .attributes
                    .iter()
                    .any(|a| a.name == AttributeName::For);

                // If the label has children, it might wrap a form control — we can't easily
                // verify this statically, so only warn if there's no `for` and no children.
                if !has_for && !element.has_children {
                    return Some(LintDiagnostic {
                        rule: Rule::LabelHasAssociatedControl,
                        message: "<label> element has no associated form control.".to_string(),
                        severity: Severity::Warning,
                        file: element.file.clone(),
                        line: element.line,
                        column: element.column,
                        element: element.tag.clone(),
                        help: Some(
                            "Add a `for` attribute linking to a form control's `id`, or nest a form control inside the label."
                                .to_string(),
                        ),
                    });
                }
            }
            Rule::Lang => {
                // Per jsx-a11y: the lang attribute must have a valid BCP 47 value.
                // This is different from html-has-lang which checks for existence.
                for attr in &element.attributes {
                    if attr.name == AttributeName::Lang {
                        if let Some(AttrValue::Static(ref val)) = attr.value {
                            if !is_valid_lang(val) {
                                return Some(LintDiagnostic {
                                    rule: Rule::Lang,
                                    message: format!(
                                        "The `lang` attribute value \"{}\" is not a valid BCP 47 language tag.",
                                        val
                                    ),
                                    severity: Severity::Error,
                                    file: element.file.clone(),
                                    line: attr.line,
                                    column: attr.column,
                                    element: element.tag.clone(),
                                    help: Some(
                                        "Use a valid BCP 47 language tag, e.g., \"en\", \"en-US\", \"fr\", \"de\", \"zh-Hans\".".to_string(),
                                    ),
                                });
                            }
                        }
                    }
                }
            }
            Rule::MediaHasCaption => {
                if !matches!(element.tag, Tag::Video | Tag::Audio) {
                    return None;
                }

                let has_accessible_text = element.attributes.iter().any(|a| {
                    a.name == AttributeName::Aria(Aria::Label)
                        || a.name == AttributeName::Aria(Aria::LabelledBy)
                });

                // We can't check for <track> children in our simplified model, but
                // we warn if there's no aria-label either, as a heuristic.
                let is_muted = element.attributes.iter().any(|a| {
                    a.name == AttributeName::Muted || a.name == AttributeName::Aria(Aria::Hidden)
                });

                if !has_accessible_text && !is_muted {
                    return Some(LintDiagnostic {
                        rule: Rule::MediaHasCaption,
                        message: format!(
                            "<{}> elements must have captions for accessibility.",
                            element.tag
                        ),
                        severity: Severity::Warning,
                        file: element.file.clone(),
                        line: element.line,
                        column: element.column,
                        element: element.tag.clone(),
                        help: Some(
                            "Add a <track kind=\"captions\"> child element, or use `aria-label` / `aria-labelledby` for descriptive text."
                                .to_string(),
                        ),
                    });
                }
            }
            Rule::MouseEventsHaveKeyEvents => {
                let mut has_mouse_enter = false;
                let mut has_mouse_leave = false;
                let mut has_on_focus = false;
                let mut has_on_blur = false;
                for attr in &element.attributes {
                    if attr.name == AttributeName::OnMouseOver {
                        has_mouse_enter = true;
                    }
                    if attr.name == AttributeName::OnMouseOut {
                        has_mouse_leave = true;
                    }
                    if attr.name == AttributeName::OnFocus {
                        has_on_focus = true;
                    }
                    if attr.name == AttributeName::OnBlur {
                        has_on_blur = true;
                    }
                }
                if has_mouse_enter && !has_on_focus {
                    return Some(LintDiagnostic {
                        rule: Rule::MouseEventsHaveKeyEvents,
                        message: format!(
                            "<{}> has a mouseover event handler but no onfocus handler. This can cause accessibility issues for keyboard users.",
                            element.tag
                        ),
                        severity: Severity::Warning,
                        file: element.file.clone(),
                        line: element.line,
                        column: element.column,
                        element: element.tag.clone(),
                        help: Some(
                            "Add an `onfocus` handler that mirrors the behavior of the `onmouseover` handler."
                                .to_string(),
                        ),
                    });
                }
                if has_mouse_leave && !has_on_blur {
                    return Some(LintDiagnostic {
                        rule: Rule::MouseEventsHaveKeyEvents,
                        message: format!(
                            "<{}> has a mouseout event handler but no onblur handler. This can cause accessibility issues for keyboard users.",
                            element.tag
                        ),
                        severity: Severity::Warning,
                        file: element.file.clone(),
                        line: element.line,
                        column: element.column,
                        element: element.tag.clone(),
                        help: Some(
                            "Add an `onblur` handler that mirrors the behavior of the `onmouseout` handler."
                                .to_string(),
                        ),
                    });
                }
            }
            Rule::NoAccessKey => {
                for attr in &element.attributes {
                    if attr.name == AttributeName::AccessKey {
                        return Some(LintDiagnostic {
                            rule: Rule::NoAccessKey,
                            message: format!(
                                "Avoid using the `accesskey` attribute on <{}>. Access keys create keyboard shortcuts that conflict with screen reader and keyboard commands.",
                                element.tag
                            ),
                            severity: Severity::Warning,
                            file: element.file.clone(),
                            line: attr.line,
                            column: attr.column,
                            element: element.tag.clone(),
                            help: None,
                        });
                    }
                }
            }
            Rule::NoAriaHiddenOnFocusable => {
                // Check if element is focusable (natively interactive or has tabindex >= 0)
                if !element.is_focusable() {
                    return None;
                }
                let has_aria_hidden_true = element.attributes.iter().any(|a| {
                    a.name == AttributeName::Aria(Aria::Hidden)
                        && matches!(&a.value, Some(AttrValue::Static(v)) if v == "true")
                });
                if has_aria_hidden_true {
                    return Some(LintDiagnostic {
                        rule: Rule::NoAriaHiddenOnFocusable,
                        message: format!(
                            "<{}> element is focusable but has `aria-hidden=\"true\"`, which hides it from assistive technologies.",
                            element.tag
                        ),
                        severity: Severity::Error,
                        file: element.file.clone(),
                        line: element.line,
                        column: element.column,
                        element: element.tag.clone(),
                        help: Some(
                            "Remove `aria-hidden=\"true\"` from focusable elements, or make the element non-focusable."
                                .to_string(),
                        ),
                    });
                }
            }
            Rule::NoAutofocus => {
                for attr in &element.attributes {
                    if attr.name == AttributeName::AutoFocus {
                        return Some(LintDiagnostic {
                            rule: Rule::NoAutofocus,
                            message: format!(
                                "Avoid using the `autofocus` attribute on <{}>. Autofocus can reduce usability and accessibility for sighted and non-sighted users.",
                                element.tag
                            ),
                            severity: Severity::Warning,
                            file: element.file.clone(),
                            line: attr.line,
                            column: attr.column,
                            element: element.tag.clone(),
                            help: None,
                        });
                    }
                }
            }
            Rule::NoDistractingElements => {
                if matches!(element.tag, Tag::Marquee | Tag::Blink) {
                    return Some(LintDiagnostic {
                        rule: Rule::NoDistractingElements,
                        message: format!(
                            "<{}> elements are distracting and should not be used. They can cause accessibility issues for users with visual or cognitive disabilities.",
                            element.tag
                        ),
                        severity: Severity::Error,
                        file: element.file.clone(),
                        line: element.line,
                        column: element.column,
                        element: element.tag.clone(),
                        help: Some("Use CSS animations or transitions instead.".to_string()),
                    });
                }
            }
            Rule::NoInteractiveElementToNoninteractiveRole => {
                // Interactive HTML elements should not be assigned non-interactive roles.
                if !element.tag.is_interactive() {
                    return None;
                }
                for attr in &element.attributes {
                    if attr.name == AttributeName::Role {
                        if let Some(AttrValue::Static(ref val)) = attr.value {
                            if let Some(role) = Role::from_str(val) {
                                if !role.is_interactive() {
                                    return Some(LintDiagnostic {
                                        rule: Rule::NoInteractiveElementToNoninteractiveRole,
                                        message: format!(
                                            "Interactive element <{}> should not be assigned the non-interactive role \"{}\".",
                                            element.tag, val
                                        ),
                                        severity: Severity::Warning,
                                        file: element.file.clone(),
                                        line: attr.line,
                                        column: attr.column,
                                        element: element.tag.clone(),
                                        help: Some(
                                            "Remove the `role` attribute or use an appropriate interactive role.".to_string(),
                                        ),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            Rule::NoNoninteractiveElementInteractions => {
                // Non-interactive elements with non-interactive implicit roles
                // should not have event handlers.
                if element.tag.is_interactive() {
                    return None;
                }

                // If the element has an explicit interactive role, it's fine.
                let explicit_role = element.attributes.iter().find_map(|a| {
                    if a.name == AttributeName::Role {
                        a.value
                            .as_ref()
                            .and_then(|v| v.as_static())
                            .and_then(Role::from_str)
                    } else {
                        None
                    }
                });
                if explicit_role.as_ref().is_some_and(|r| r.is_interactive()) {
                    return None;
                }

                // Only applies to elements with a non-interactive implicit role
                // (elements with no implicit role are handled by NoStaticElementInteractions)
                if element.tag.implicit_role().is_none() {
                    return None;
                }

                let has_handler = element.attributes.iter().any(|a| {
                    matches!(
                        a.name,
                        AttributeName::OnClick
                            | AttributeName::OnKeyDown
                            | AttributeName::OnKeyUp
                            | AttributeName::OnKeyPress
                    )
                });

                if has_handler {
                    return Some(LintDiagnostic {
                        rule: Rule::NoNoninteractiveElementInteractions,
                        message: format!(
                            "Non-interactive element <{}> should not have event handlers.",
                            element.tag
                        ),
                        severity: Severity::Warning,
                        file: element.file.clone(),
                        line: element.line,
                        column: element.column,
                        element: element.tag.clone(),
                        help: Some(
                            "Use an interactive element like <button> or <a>, or add an appropriate `role` attribute."
                                .to_string(),
                        ),
                    });
                }
            }
            Rule::NoNoninteractiveElementToInteractiveRole => {
                // Non-interactive HTML elements should not be assigned interactive roles.
                if element.tag.is_interactive() {
                    return None;
                }
                for attr in &element.attributes {
                    if attr.name == AttributeName::Role {
                        if let Some(AttrValue::Static(ref val)) = attr.value {
                            if let Some(role) = Role::from_str(val) {
                                if role.is_interactive() {
                                    return Some(LintDiagnostic {
                                        rule: Rule::NoNoninteractiveElementToInteractiveRole,
                                        message: format!(
                                            "Non-interactive element <{}> should not be assigned the interactive role \"{}\".",
                                            element.tag, val
                                        ),
                                        severity: Severity::Warning,
                                        file: element.file.clone(),
                                        line: attr.line,
                                        column: attr.column,
                                        element: element.tag.clone(),
                                        help: Some(
                                            "Use the appropriate interactive element instead, e.g., <button>, <a>, <input>."
                                                .to_string(),
                                        ),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            Rule::NoNoninteractiveTabindex => {
                if element.role().is_some_and(|role| role.is_interactive()) {
                    return None;
                }

                for attr in &element.attributes {
                    if attr.name == AttributeName::TabIndex {
                        if let Some(AttrValue::Static(ref val)) = attr.value {
                            if let Ok(index) = val.parse::<i32>() {
                                if index >= 0 {
                                    return Some(LintDiagnostic {
                                        rule: Rule::NoNoninteractiveTabindex,
                                        message: format!(
                                            "Non-interactive element <{}> should not have `tabindex=\"{}\"`. Non-interactive elements should not be focusable.",
                                            element.tag, index
                                        ),
                                        severity: Severity::Warning,
                                        file: element.file.clone(),
                                        line: attr.line,
                                        column: attr.column,
                                        element: element.tag.clone(),
                                        help: Some(
                                            "Remove the `tabindex` attribute, or add an interactive role."
                                                .to_string(),
                                        ),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            Rule::NoRedundantRoles => {
                if let Some(implicit_role) = element.tag.implicit_role() {
                    for attr in &element.attributes {
                        if attr.name == AttributeName::Role {
                            if let Some(AttrValue::Static(ref val)) = attr.value {
                                if Role::from_str(val) == Some(implicit_role.clone()) {
                                    return Some(LintDiagnostic {
                                        rule: Rule::NoRedundantRoles,
                                        message: format!(
                                            "Redundant role \"{}\" on <{}>. This is the element's implicit role.",
                                            val, element.tag
                                        ),
                                        severity: Severity::Warning,
                                        file: element.file.clone(),
                                        line: attr.line,
                                        column: attr.column,
                                        element: element.tag.clone(),
                                        help: Some("Remove the `role` attribute.".to_string()),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            Rule::NoStaticElementInteractions => {
                // Static HTML elements (no implicit role) with event handlers
                // should have a `role` attribute.
                if element.tag.is_interactive() || element.tag.implicit_role().is_some() {
                    return None;
                }

                // If element has an explicit role, it's no longer "static"
                let has_role = element
                    .attributes
                    .iter()
                    .any(|a| a.name == AttributeName::Role);
                if has_role {
                    return None;
                }

                let has_handler = element.attributes.iter().any(|a| {
                    matches!(
                        a.name,
                        AttributeName::OnClick
                            | AttributeName::OnKeyDown
                            | AttributeName::OnKeyUp
                            | AttributeName::OnKeyPress
                            | AttributeName::OnMouseOver
                            | AttributeName::OnMouseOut
                    )
                });

                if has_handler {
                    return Some(LintDiagnostic {
                        rule: Rule::NoStaticElementInteractions,
                        message: format!(
                            "<{}> with event handler(s) must have a `role` attribute.",
                            element.tag
                        ),
                        severity: Severity::Warning,
                        file: element.file.clone(),
                        line: element.line,
                        column: element.column,
                        element: element.tag.clone(),
                        help: Some(
                            "Add a `role` attribute that describes the element's purpose, or use a semantic element like <button> or <a>."
                                .to_string(),
                        ),
                    });
                }
            }
            Rule::PreferTagOverRole => {
                for attr in &element.attributes {
                    if attr.name == AttributeName::Role {
                        if let Some(AttrValue::Static(ref val)) = attr.value {
                            if let Some(role) = Role::from_str(val) {
                                if let Some(preferred) = role.preferred_tag() {
                                    // Don't flag if the element already IS the preferred tag
                                    if element.tag.implicit_role().as_ref() == Some(&role) {
                                        return None;
                                    }
                                    return Some(LintDiagnostic {
                                        rule: Rule::PreferTagOverRole,
                                        message: format!(
                                            "Prefer using the {} element instead of `role=\"{}\"`.",
                                            preferred, val
                                        ),
                                        severity: Severity::Info,
                                        file: element.file.clone(),
                                        line: attr.line,
                                        column: attr.column,
                                        element: element.tag.clone(),
                                        help: Some(format!(
                                            "Use {0} which has built-in semantics and keyboard behavior instead of relying on ARIA.",
                                            preferred
                                        )),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            Rule::RoleHasRequiredAriaProps => {
                let role_attr = element
                    .attributes
                    .iter()
                    .find(|a| a.name == AttributeName::Role);
                if let Some(role_attr) = role_attr {
                    if let Some(AttrValue::Static(ref val)) = role_attr.value {
                        if let Some(role) = Role::from_str(val) {
                            let required = role.required_aria_props();
                            if required.is_empty() {
                                return None;
                            }

                            // For heading role on h1-h6, level is implicit
                            if role == Role::Heading && element.tag.is_heading() {
                                return None;
                            }

                            let missing: Vec<&Aria> = required
                                .iter()
                                .filter(|req| {
                                    !element
                                        .attributes
                                        .iter()
                                        .any(|a| a.name == AttributeName::Aria((*req).clone()))
                                })
                                .collect();

                            if !missing.is_empty() {
                                let missing_names: Vec<String> =
                                    missing.iter().map(|a| format!("`{}`", a)).collect();
                                return Some(LintDiagnostic {
                                    rule: Rule::RoleHasRequiredAriaProps,
                                    message: format!(
                                        "<{}> with role=\"{}\" is missing required ARIA properties: {}.",
                                        element.tag,
                                        val,
                                        missing_names.join(", ")
                                    ),
                                    severity: Severity::Error,
                                    file: element.file.clone(),
                                    line: role_attr.line,
                                    column: role_attr.column,
                                    element: element.tag.clone(),
                                    help: Some(format!(
                                        "Add the required ARIA properties for the \"{}\" role.",
                                        val
                                    )),
                                });
                            }
                        }
                    }
                }
            }
            Rule::RoleSupportsAriaProps => {
                // Determine the effective role
                let effective_role = element.role();
                if let Some(role) = effective_role {
                    for attr in &element.attributes {
                        if let AttributeName::Aria(ref aria) = attr.name {
                            if !aria.is_supported_by_role(&role) {
                                return Some(LintDiagnostic {
                                    rule: Rule::RoleSupportsAriaProps,
                                    message: format!(
                                        "The `{}` property is not supported by the \"{}\" role on <{}>.",
                                        attr.name, role, element.tag
                                    ),
                                    severity: Severity::Warning,
                                    file: element.file.clone(),
                                    line: attr.line,
                                    column: attr.column,
                                    element: element.tag.clone(),
                                    help: Some(format!(
                                        "Remove the `{}` property, or change the role to one that supports it.",
                                        attr.name
                                    )),
                                });
                            }
                        }
                    }
                }
            }
            Rule::Scope => {
                if element.tag == Tag::Th {
                    return None; // scope on <th> is fine
                }

                for attr in &element.attributes {
                    if attr.name == AttributeName::Scope {
                        return Some(LintDiagnostic {
                            rule: Rule::Scope,
                            message: format!(
                                "The `scope` attribute should only be used on <th> elements, not <{}>.",
                                element.tag
                            ),
                            severity: Severity::Warning,
                            file: element.file.clone(),
                            line: attr.line,
                            column: attr.column,
                            element: element.tag.clone(),
                            help: None,
                        });
                    }
                }
            }
            Rule::TabindexNoPositive => {
                for attr in &element.attributes {
                    if attr.name == AttributeName::TabIndex {
                        if let Some(AttrValue::Static(ref val)) = attr.value {
                            if let Ok(index) = val.parse::<i32>() {
                                if index > 0 {
                                    return Some(LintDiagnostic {
                                        rule: Rule::TabindexNoPositive,
                                        message: format!(
                                            "Avoid using positive `tabindex` value ({}) on <{}>. This creates an unexpected tab order.",
                                            index, element.tag
                                        ),
                                        severity: Severity::Warning,
                                        file: element.file.clone(),
                                        line: attr.line,
                                        column: attr.column,
                                        element: element.tag.clone(),
                                        help: Some(
                                            "Use `tabindex=\"0\"` for focusable elements or `tabindex=\"-1\"` for programmatically focusable elements."
                                                .to_string(),
                                        ),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

/// A lint diagnostic produced by a lint rule.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct LintDiagnostic {
    /// Unique identifier for the lint rule (e.g., "invalid-aria-attribute").
    pub rule: Rule,
    /// Human-readable message.
    pub message: String,
    /// Severity level.
    pub severity: Severity,
    /// File where the issue was found.
    pub file: String,
    /// Line number (1-based).
    pub line: usize,
    /// Column number (0-based).
    pub column: usize,
    /// The element name where the issue was found.
    pub element: Tag,
    /// Optional help text for fixing the issue.
    pub help: Option<String>,
}

/// Run all lint rules on a collection of parsed HTML elements.
///
/// Returns a lazy iterator — no allocation occurs until the caller collects
/// or consumes the diagnostics.
pub fn run_all_lints(elements: &[HtmlElement]) -> impl Iterator<Item = LintDiagnostic> + '_ {
    elements
        .iter()
        .flat_map(|element| Rule::iter().filter_map(move |rule| rule.check(element)))
}

// ---------------------------------------------------------------------------
// Helper functions for lint rules
// ---------------------------------------------------------------------------

/// Validate an autocomplete attribute value per the HTML spec.
fn is_valid_autocomplete(value: &str) -> bool {
    const VALID_TOKENS: &[&str] = &[
        "on",
        "off",
        "name",
        "honorific-prefix",
        "given-name",
        "additional-name",
        "family-name",
        "honorific-suffix",
        "nickname",
        "email",
        "username",
        "new-password",
        "current-password",
        "one-time-code",
        "organization-title",
        "organization",
        "street-address",
        "address-line1",
        "address-line2",
        "address-line3",
        "address-level4",
        "address-level3",
        "address-level2",
        "address-level1",
        "country",
        "country-name",
        "postal-code",
        "cc-name",
        "cc-given-name",
        "cc-additional-name",
        "cc-family-name",
        "cc-number",
        "cc-exp",
        "cc-exp-month",
        "cc-exp-year",
        "cc-csc",
        "cc-type",
        "transaction-currency",
        "transaction-amount",
        "language",
        "bday",
        "bday-day",
        "bday-month",
        "bday-year",
        "sex",
        "tel",
        "tel-country-code",
        "tel-national",
        "tel-area-code",
        "tel-local",
        "tel-extension",
        "impp",
        "url",
        "photo",
        "webauthn",
    ];
    const SECTION_PREFIXES: &[&str] = &["shipping", "billing"];

    let tokens: Vec<&str> = value.split_whitespace().collect();
    if tokens.is_empty() {
        return false;
    }

    let mut idx = 0;

    // Optional section-* prefix
    if let Some(token) = tokens.get(idx) {
        if token.starts_with("section-") {
            idx += 1;
        }
    }

    // Optional shipping/billing
    if let Some(token) = tokens.get(idx) {
        if SECTION_PREFIXES.contains(token) {
            idx += 1;
        }
    }

    // Must have at least the field token
    if idx >= tokens.len() {
        return false;
    }

    // Remaining token must be a valid field name
    let field = tokens[idx];
    idx += 1;

    // Must be the last token
    if idx != tokens.len() {
        return false;
    }

    VALID_TOKENS.contains(&field)
}

/// Validate a BCP 47 language tag (simplified check per jsx-a11y lang rule).
fn is_valid_lang(lang: &str) -> bool {
    let lang = lang.trim();
    if lang.is_empty() {
        return false;
    }

    let parts: Vec<&str> = lang.split('-').collect();
    let primary = parts[0];

    // Primary language subtag must be 2-3 ASCII letters
    if primary.len() < 2 || primary.len() > 3 {
        return false;
    }
    if !primary.chars().all(|c| c.is_ascii_alphabetic()) {
        return false;
    }

    // All remaining subtags must be non-empty and alphanumeric
    for part in &parts[1..] {
        if part.is_empty() || part.len() > 8 {
            return false;
        }
        if !part.chars().all(|c| c.is_ascii_alphanumeric()) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
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
        let diags =
            lint_source(r#"fn c() { html! { <img src="test.png" role="presentation" /> } }"#);
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
        let diags =
            lint_source(r#"fn c() { html! { <img src="test.png" alt="image of a cat" /> } }"#);
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
        let diags =
            lint_source(r#"fn c() { html! { <div role="button" onclick={handler}></div> } }"#);
        assert!(has_lint(&diags, Rule::ClickEventsHaveKeyEvents));
    }

    #[test]
    fn test_click_on_button_ok() {
        let diags =
            lint_source(r#"fn c() { html! { <button onclick={handler}>{"Click"}</button> } }"#);
        assert!(!has_lint(&diags, Rule::ClickEventsHaveKeyEvents));
    }

    #[test]
    fn test_leptos_view_macro() {
        let diags = lint_source(r#"fn c() { view! { <img src="test.png" /> } }"#);
        assert!(has_lint(&diags, Rule::AltText));
    }

    #[test]
    fn test_iframe_missing_title() {
        let diags =
            lint_source(r#"fn c() { html! { <iframe src="https://example.com"></iframe> } }"#);
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
        let diags =
            lint_source(r#"fn c() { html! { <a href="/about" aria-label="About us"></a> } }"#);
        assert!(!has_lint(&diags, Rule::AnchorHasContent));
    }

    // --- AriaActivedescendantHasTabindex ---

    #[test]
    fn test_activedescendant_without_tabindex() {
        let diags =
            lint_source(r#"fn c() { html! { <div aria-activedescendant="item1"></div> } }"#);
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
        let diags =
            lint_source(r#"fn c() { html! { <div role="button" onclick={handler}></div> } }"#);
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
        let diags = lint_source(
            r#"fn c() { html! { <video src="v.mp4" aria-label="Tutorial"></video> } }"#,
        );
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
        let diags = lint_source(
            r#"fn c() { html! { <div onmouseover={handler} onfocus={handler}></div> } }"#,
        );
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
        let diags =
            lint_source(r#"fn c() { html! { <button aria-hidden="true">{"x"}</button> } }"#);
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
        let diags = lint_source(
            r#"fn c() { html! { <article role="button" onclick={handler}></article> } }"#,
        );
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
        let diags =
            lint_source(r#"fn c() { html! { <div role="button" onclick={handler}></div> } }"#);
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
        let diags =
            lint_source(r#"fn c() { html! { <div role="alert" aria-checked="true"></div> } }"#);
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
}
