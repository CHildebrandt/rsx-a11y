//! Lint rules for ARIA and accessibility checks.
//!
//! Each [`Rule`] checks one accessibility concern on the [`HtmlElement`]s
//! found in Yew/Leptos/Dioxus macro invocations. Use [`run_all_lints`] to run
//! every rule over a set of elements.

mod rules;
#[cfg(test)]
mod tests;
mod values;

use std::collections::{HashMap, HashSet};

use crate::dom::{AttributeName, Tag, serde_via_str};
use crate::parser::{HtmlAttribute, HtmlElement};
use strum::IntoEnumIterator;

/// Severity level for a lint diagnostic.
///
/// Severities are ordered: `Info < Warning < Error`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// A suggestion for improved accessibility.
    Info,
    /// A likely problem that should be reviewed.
    Warning,
    /// A violation that must be fixed (e.g. missing `alt` on `<img>`).
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Error => "error",
        })
    }
}

/// Accessibility lint rule identifiers.
///
/// Each variant corresponds to a single lint check. Rules are named in
/// `kebab-case` (e.g. `alt-text`, `aria-role`) in CLI flags, JSON output,
/// [`Display`](std::fmt::Display) and [`FromStr`](std::str::FromStr).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::EnumIter,
    strum::VariantArray,
    strum::EnumString,
    strum::Display,
    strum::IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
#[non_exhaustive]
pub enum Rule {
    /// Elements that need alternative text (`<img>`, `<area>`, `<input type="image">`, `<object>`) must have it.
    AltText,
    /// Link text must not be generic, such as "click here" or "learn more".
    AnchorAmbiguousText,
    /// Links must have discernible content.
    AnchorHasContent,
    /// Links must have a real, navigable `href`.
    AnchorIsValid,
    /// Elements managing `aria-activedescendant` must be focusable.
    AriaActivedescendantHasTabindex,
    /// `aria-*` attributes must be valid WAI-ARIA properties.
    AriaProps,
    /// ARIA property values must have the right type.
    AriaProptypes,
    /// `role` values must be valid, non-abstract WAI-ARIA roles.
    AriaRole,
    /// Elements that do not support ARIA must not carry ARIA attributes.
    AriaUnsupportedElements,
    /// `autocomplete` values must be valid.
    AutocompleteValid,
    /// Click handlers on non-interactive elements need a keyboard equivalent.
    ClickEventsHaveKeyEvents,
    /// Form controls must have an accessible label.
    ControlHasAssociatedLabel,
    /// Headings must have content.
    HeadingHasContent,
    /// `<html>` must have a `lang` attribute.
    HtmlHasLang,
    /// `<iframe>` must have a title.
    IframeHasTitle,
    /// `<img>` alt text must not contain words like "image" or "photo".
    ImgRedundantAlt,
    /// Elements with interactive roles and handlers must be focusable.
    InteractiveSupportsFocus,
    /// `<label>` must be associated with a control.
    LabelHasAssociatedControl,
    /// `lang` values must be valid BCP 47 language tags.
    Lang,
    /// `<audio>` and `<video>` must have captions.
    MediaHasCaption,
    /// Mouse hover handlers need focus/blur equivalents.
    MouseEventsHaveKeyEvents,
    /// Do not use `accesskey`.
    NoAccessKey,
    /// Focusable elements must not be `aria-hidden`.
    NoAriaHiddenOnFocusable,
    /// Do not use `autofocus`.
    NoAutofocus,
    /// Do not use `<marquee>` or `<blink>`.
    NoDistractingElements,
    /// Interactive elements must not get non-interactive roles.
    NoInteractiveElementToNoninteractiveRole,
    /// Non-interactive elements must not have interaction handlers.
    NoNoninteractiveElementInteractions,
    /// Non-interactive elements must not get interactive roles.
    NoNoninteractiveElementToInteractiveRole,
    /// Non-interactive elements must not be in the tab order.
    NoNoninteractiveTabindex,
    /// Do not repeat an element's implicit role.
    NoRedundantRoles,
    /// Static elements with handlers need a role.
    NoStaticElementInteractions,
    /// Prefer semantic elements over ARIA roles.
    PreferTagOverRole,
    /// Roles must have their required ARIA properties.
    RoleHasRequiredAriaProps,
    /// ARIA properties must be supported by the element's role.
    RoleSupportsAriaProps,
    /// `scope` is only valid on `<th>`.
    Scope,
    /// Do not use positive `tabindex` values.
    TabindexNoPositive,
}

serde_via_str!(Rule);

impl Rule {
    /// The rule's kebab-case identifier, e.g. `"alt-text"`.
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    /// The severity of diagnostics produced by this rule.
    pub const fn severity(self) -> Severity {
        match self {
            Rule::AltText
            | Rule::AriaProps
            | Rule::AriaProptypes
            | Rule::AriaRole
            | Rule::AriaUnsupportedElements
            | Rule::AutocompleteValid
            | Rule::Lang
            | Rule::NoAriaHiddenOnFocusable
            | Rule::NoDistractingElements
            | Rule::RoleHasRequiredAriaProps => Severity::Error,
            Rule::PreferTagOverRole => Severity::Info,
            _ => Severity::Warning,
        }
    }

    /// Iterate over every rule.
    pub fn all() -> impl Iterator<Item = Rule> {
        Rule::iter()
    }

    /// One-sentence description of what the rule enforces.
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
            Rule::AutocompleteValid => "Enforce that autocomplete attributes are used correctly.",
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

    /// WCAG success criteria ("Understanding" pages) the rule relates to.
    pub const fn guidelines(&self) -> &'static [&'static str] {
        match self {
            Rule::AltText => &["https://www.w3.org/WAI/WCAG21/Understanding/non-text-content.html"],
            Rule::AnchorAmbiguousText => &[],
            Rule::AnchorHasContent => &[
                "https://www.w3.org/WAI/WCAG21/Understanding/link-purpose-in-context",
                "https://www.w3.org/WAI/WCAG21/Understanding/name-role-value",
            ],
            Rule::AnchorIsValid => &["https://www.w3.org/WAI/WCAG21/Understanding/keyboard"],
            Rule::AriaActivedescendantHasTabindex => {
                &["https://www.w3.org/WAI/WCAG21/Understanding/keyboard"]
            }
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

    /// Further reading about the rule.
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

    /// Run this rule against one element of `doc`, appending any
    /// diagnostics to `out`.
    fn check(self, element: &HtmlElement, doc: &Document<'_>, out: &mut Vec<LintDiagnostic>) {
        let mut cx = Emitter {
            rule: self,
            element,
            doc,
            out,
        };
        rules::check(self, element, &mut cx);
    }
}

/// A lint diagnostic produced by a lint rule.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct LintDiagnostic {
    /// The rule that produced this diagnostic (serialized as e.g. `"alt-text"`).
    pub rule: Rule,
    /// Human-readable message.
    pub message: String,
    /// Severity level.
    pub severity: Severity,
    /// File where the issue was found.
    pub file: String,
    /// Line number (1-based).
    pub line: usize,
    /// Column number (1-based).
    pub column: usize,
    /// The element on which the issue was found.
    pub element: Tag,
    /// Help text describing how to fix the issue.
    pub help: Option<String>,
}

/// Run all lint rules on the elements of one file (as returned by
/// [`parse_source`](crate::parser::parse_source)).
///
/// Diagnostics are returned in element order, then rule order. Some rules
/// look at related elements (a `<label for>` elsewhere in the file, a
/// `<track>` child), so pass all of a file's elements together.
pub fn run_all_lints(elements: &[HtmlElement]) -> impl Iterator<Item = LintDiagnostic> + '_ {
    run_lints(elements, Rule::iter())
}

/// Run the given rules on the elements of one file.
pub fn run_lints(
    elements: &[HtmlElement],
    rules: impl IntoIterator<Item = Rule>,
) -> impl Iterator<Item = LintDiagnostic> + '_ {
    let rules: Vec<Rule> = rules.into_iter().collect();
    let doc = Document::new(elements);
    let mut out = Vec::new();
    for element in elements {
        for &rule in &rules {
            rule.check(element, &doc, &mut out);
        }
    }
    out.into_iter()
}

/// Relationships between the elements of one file.
pub(crate) struct Document<'a> {
    by_index: HashMap<usize, &'a HtmlElement>,
    children: HashMap<usize, Vec<&'a HtmlElement>>,
    /// Static `for` values of `<label>` elements.
    label_targets: HashSet<&'a str>,
    /// Whether some `<label>` has a dynamic `for`, which could target anything.
    has_dynamic_label_target: bool,
}

impl<'a> Document<'a> {
    fn new(elements: &'a [HtmlElement]) -> Self {
        let mut doc = Document {
            by_index: HashMap::new(),
            children: HashMap::new(),
            label_targets: HashSet::new(),
            has_dynamic_label_target: false,
        };
        for el in elements {
            doc.by_index.insert(el.index, el);
            if let Some(parent) = el.parent {
                doc.children.entry(parent).or_default().push(el);
            }
            if el.tag == Tag::Label
                && let Some(attr) = el.attr(&AttributeName::For)
            {
                match attr.value.as_static() {
                    Some(id) => {
                        doc.label_targets.insert(id.trim());
                    }
                    None => doc.has_dynamic_label_target = true,
                }
            }
        }
        doc
    }

    /// Direct child elements of `el`.
    pub(crate) fn children(&self, el: &HtmlElement) -> &[&'a HtmlElement] {
        self.children.get(&el.index).map_or(&[], Vec::as_slice)
    }

    /// All descendant elements of `el`.
    pub(crate) fn descendants(&self, el: &HtmlElement) -> Vec<&'a HtmlElement> {
        let mut out = Vec::new();
        let mut stack: Vec<&HtmlElement> = self.children(el).to_vec();
        while let Some(next) = stack.pop() {
            out.push(next);
            stack.extend(self.children(next));
        }
        out
    }

    /// Enclosing elements of `el`, nearest first.
    pub(crate) fn ancestors(&self, el: &HtmlElement) -> impl Iterator<Item = &'a HtmlElement> {
        std::iter::successors(
            el.parent.and_then(|p| self.by_index.get(&p).copied()),
            |e| e.parent.and_then(|p| self.by_index.get(&p).copied()),
        )
    }

    /// Whether `el` is referenced by a `<label for>` in this file, or might be.
    pub(crate) fn is_label_target(&self, el: &HtmlElement) -> bool {
        match el.attr(&AttributeName::Id).map(|a| a.value.as_static()) {
            None => false,
            Some(None) => true, // dynamic id
            Some(Some(id)) => {
                self.has_dynamic_label_target || self.label_targets.contains(id.trim())
            }
        }
    }
}

/// Builds diagnostics for one rule on one element.
pub(crate) struct Emitter<'a> {
    rule: Rule,
    element: &'a HtmlElement,
    pub(crate) doc: &'a Document<'a>,
    out: &'a mut Vec<LintDiagnostic>,
}

impl Emitter<'_> {
    fn push(&mut self, line: usize, column: usize, message: String, help: String) {
        self.out.push(LintDiagnostic {
            rule: self.rule,
            message,
            severity: self.rule.severity(),
            file: self.element.file.clone(),
            line,
            column,
            element: self.element.tag,
            help: Some(help),
        });
    }

    /// Report an issue located at the element's opening tag.
    pub(crate) fn on_element(&mut self, message: impl Into<String>, help: impl Into<String>) {
        let (line, column) = (self.element.line, self.element.column);
        self.push(line, column, message.into(), help.into());
    }

    /// Report an issue located at a specific attribute.
    pub(crate) fn on_attr(
        &mut self,
        attr: &HtmlAttribute,
        message: impl Into<String>,
        help: impl Into<String>,
    ) {
        self.push(attr.line, attr.column, message.into(), help.into());
    }
}
