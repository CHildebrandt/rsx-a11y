//! rsx-a11y: A linting tool for ARIA and accessibility attributes
//! in Rust web frameworks (Yew, Leptos, Dioxus).
//!
//! This crate parses Rust source files, finds macro invocations containing
//! HTML-like RSX content (via [`rstml`](https://docs.rs/rstml)), and checks
//! for accessibility issues based on the WAI-ARIA 1.2 specification.
//!
//! # Supported Lints (36)
//!
//! ## Errors (10)
//!
//! | Lint ID | Description |
//! |---------|-------------|
//! | `alt-text` | Elements requiring alt text (`<img>`, `<area>`, `<input type="image">`, `<object>`) must have it |
//! | `aria-props` | Unknown `aria-*` attribute |
//! | `aria-proptypes` | Invalid value for a known ARIA attribute |
//! | `aria-role` | Unknown or abstract WAI-ARIA role |
//! | `aria-unsupported-elements` | ARIA on elements that don't support it |
//! | `autocomplete-valid` | Invalid `autocomplete` attribute value |
//! | `lang` | Invalid BCP 47 language tag |
//! | `no-aria-hidden-on-focusable` | `aria-hidden="true"` on a focusable element |
//! | `no-distracting-elements` | `<marquee>` or `<blink>` used |
//! | `role-has-required-aria-props` | Missing required ARIA properties for a given role |
//!
//! ## Warnings (25)
//!
//! | Lint ID | Description |
//! |---------|-------------|
//! | `anchor-ambiguous-text` | `<a>` text must not be generic ("click here", "here", etc.) |
//! | `anchor-has-content` | `<a>` without discernible text |
//! | `anchor-is-valid` | `<a>` with `href="#"`, empty `href`, or `javascript:void(0)` |
//! | `aria-activedescendant-has-tabindex` | Non-interactive element with `aria-activedescendant` needs `tabindex` |
//! | `click-events-have-key-events` | Click handler without keyboard handler on non-interactive element |
//! | `control-has-associated-label` | Interactive controls must have a text label |
//! | `heading-has-content` | Empty heading element |
//! | `html-has-lang` | `<html>` without `lang` attribute |
//! | `iframe-has-title` | `<iframe>` without `title` |
//! | `img-redundant-alt` | `<img>` alt text contains "image", "picture", "photo" |
//! | `interactive-supports-focus` | Element with interactive role and event handler must be focusable |
//! | `label-has-associated-control` | `<label>` without associated form control |
//! | `media-has-caption` | `<video>` or `<audio>` without captions |
//! | `mouse-events-have-key-events` | `onmouseover`/`onmouseout` without `onfocus`/`onblur` |
//! | `no-access-key` | `accesskey` attribute used |
//! | `no-autofocus` | `autofocus` attribute used |
//! | `no-interactive-element-to-noninteractive-role` | Interactive element assigned a non-interactive role |
//! | `no-noninteractive-element-interactions` | Non-interactive element with event handlers |
//! | `no-noninteractive-element-to-interactive-role` | Non-interactive element assigned an interactive role |
//! | `no-noninteractive-tabindex` | `tabindex` on non-interactive element |
//! | `no-redundant-roles` | Explicit role matches element's implicit role |
//! | `no-static-element-interactions` | Static element with event handlers but no role |
//! | `role-supports-aria-props` | ARIA property not supported by the element's role |
//! | `scope` | `scope` on non-`<th>` element |
//! | `tabindex-no-positive` | `tabindex` > 0 |
//!
//! ## Info (1)
//!
//! | Lint ID | Description |
//! |---------|-------------|
//! | `prefer-tag-over-role` | Prefer semantic HTML element over ARIA role |

pub mod diagnostics;
pub mod dom;
pub mod lints;
pub mod parser;
pub mod prelude;

use std::path::{Path, PathBuf};

use lints::LintDiagnostic;
use parser::ParseError;

/// Summary returned by [`check_project`] containing every diagnostic found,
/// any parse errors, and the number of files that contained lintable elements.
///
/// No filtering is applied — callers can filter `diagnostics` by
/// [`Rule`](lints::Rule), [`Severity`](lints::Severity), file path, etc.
/// after the fact.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LintSummary {
    /// All lint diagnostics found, sorted by file → line → column.
    pub diagnostics: Vec<LintDiagnostic>,
    /// Files that could not be parsed.
    pub parse_errors: Vec<ParseError>,
    /// Number of files that contained at least one lintable RSX element.
    pub files_checked: usize,
}

/// Lint an entire project (or single file) at `path` and return a [`LintSummary`].
///
/// This is the primary entry point for programmatic / unit-test usage.
/// It discovers all `.rs` files under `path` (skipping `target/`,
/// `node_modules/`, and dot-directories), parses each file for RSX macros,
/// and runs every lint rule.
///
/// # Example
///
/// ```rust,no_run
/// use std::path::Path;
/// use rsx_a11y::{check_project, LintSummary};
/// use rsx_a11y::lints::Severity;
///
/// let summary = check_project(Path::new("."));
/// let errors: Vec<_> = summary.diagnostics
///     .iter()
///     .filter(|d| d.severity == Severity::Error)
///     .collect();
/// assert!(errors.is_empty(), "accessibility errors found: {errors:#?}");
/// ```
pub fn check_project(path: &Path) -> LintSummary {
    let rust_files = collect_rust_files(path);
    let mut diagnostics: Vec<LintDiagnostic> = Vec::new();
    let mut parse_errors: Vec<ParseError> = Vec::new();
    let mut files_checked: usize = 0;

    for file in &rust_files {
        match parser::parse_file(file) {
            Ok(elements) => {
                if !elements.is_empty() {
                    files_checked += 1;
                    diagnostics.extend(lints::run_all_lints(&elements));
                }
            }
            Err(e) => parse_errors.push(e),
        }
    }

    diagnostics.sort_unstable_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then(a.line.cmp(&b.line))
            .then(a.column.cmp(&b.column))
    });

    LintSummary {
        diagnostics,
        parse_errors,
        files_checked,
    }
}

/// Recursively collect `.rs` files from `path`, skipping common non-source
/// directories (`target/`, `node_modules/`, dot-directories).
fn collect_rust_files(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        return if path.extension().is_some_and(|ext| ext == "rs") {
            vec![path.to_path_buf()]
        } else {
            Vec::new()
        };
    }

    let mut files = Vec::new();
    collect_rust_files_recursive(path, &mut files);
    files
}

fn collect_rust_files_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if matches!(name.as_ref(), "target" | "node_modules") || name.starts_with('.') {
                continue;
            }
            collect_rust_files_recursive(&path, out);
        } else if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}
