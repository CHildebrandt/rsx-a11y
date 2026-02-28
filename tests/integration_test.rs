//! Integration tests for rsx-a11y.

use std::path::Path;

use rsx_a11y::lints::{self, LintDiagnostic, Rule, Severity};
use rsx_a11y::parser;
use rsx_a11y::{check_project};

fn lint_fixture(filename: &str) -> Vec<LintDiagnostic> {
    let path = format!("tests/fixtures/{}", filename);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path, e));
    let elements = parser::parse_source(&source, &path).unwrap();
    lints::run_all_lints(&elements).collect()
}

fn count_lint(diags: &[LintDiagnostic], lint_id: Rule) -> usize {
    diags.iter().filter(|d| d.rule == lint_id).count()
}

fn has_lint(diags: &[LintDiagnostic], lint_id: Rule) -> bool {
    count_lint(diags, lint_id) > 0
}

// --- Yew fixture tests ---

#[test]
fn test_yew_fixture_has_issues() {
    let diags = lint_fixture("yew_component.rs");
    assert!(
        !diags.is_empty(),
        "Expected lint diagnostics from yew fixture"
    );
}

#[test]
fn test_yew_invalid_aria_attribute_detected() {
    let diags = lint_fixture("yew_component.rs");
    assert!(has_lint(&diags, Rule::AriaProps));
}

#[test]
fn test_yew_missing_alt_text_detected() {
    let diags = lint_fixture("yew_component.rs");
    assert!(has_lint(&diags, Rule::AltText));
}

#[test]
fn test_yew_invalid_aria_value_detected() {
    let diags = lint_fixture("yew_component.rs");
    assert!(has_lint(&diags, Rule::AriaProptypes));
}

#[test]
fn test_yew_invalid_role_detected() {
    let diags = lint_fixture("yew_component.rs");
    assert!(has_lint(&diags, Rule::AriaRole));
}

#[test]
fn test_yew_redundant_role_detected() {
    let diags = lint_fixture("yew_component.rs");
    assert!(has_lint(&diags, Rule::NoRedundantRoles));
}

#[test]
fn test_yew_no_access_key_detected() {
    let diags = lint_fixture("yew_component.rs");
    assert!(has_lint(&diags, Rule::NoAccessKey));
}

#[test]
fn test_yew_no_autofocus_detected() {
    let diags = lint_fixture("yew_component.rs");
    assert!(has_lint(&diags, Rule::NoAutofocus));
}

#[test]
fn test_yew_click_without_keyboard_detected() {
    let diags = lint_fixture("yew_component.rs");
    assert!(has_lint(&diags, Rule::ClickEventsHaveKeyEvents));
}

#[test]
fn test_yew_no_distracting_elements_detected() {
    let diags = lint_fixture("yew_component.rs");
    assert!(has_lint(&diags, Rule::NoDistractingElements));
}

#[test]
fn test_yew_anchor_is_valid_detected() {
    let diags = lint_fixture("yew_component.rs");
    assert!(has_lint(&diags, Rule::AnchorIsValid));
}

#[test]
fn test_yew_no_redundant_alt_detected() {
    let diags = lint_fixture("yew_component.rs");
    assert!(has_lint(&diags, Rule::ImgRedundantAlt));
}

#[test]
fn test_yew_iframe_has_title_detected() {
    let diags = lint_fixture("yew_component.rs");
    assert!(has_lint(&diags, Rule::IframeHasTitle));
}

#[test]
fn test_yew_has_errors() {
    let diags = lint_fixture("yew_component.rs");
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(!errors.is_empty(), "Expected errors in yew fixture");
}

// --- Leptos fixture tests ---

#[test]
fn test_leptos_fixture_has_issues() {
    let diags = lint_fixture("leptos_component.rs");
    assert!(
        !diags.is_empty(),
        "Expected lint diagnostics from leptos fixture"
    );
}

#[test]
fn test_leptos_missing_alt_text_detected() {
    let diags = lint_fixture("leptos_component.rs");
    assert!(has_lint(&diags, Rule::AltText));
}

#[test]
fn test_leptos_invalid_role_detected() {
    let diags = lint_fixture("leptos_component.rs");
    assert!(has_lint(&diags, Rule::AriaRole));
}

// --- check_project tests ---

#[test]
fn test_check_project_finds_fixtures() {
    let summary = check_project(Path::new("tests/fixtures"));

    assert!(
        summary.files_checked >= 2,
        "Expected at least 2 fixture files with lintable elements, got {}",
        summary.files_checked
    );
    assert!(
        !summary.diagnostics.is_empty(),
        "Expected diagnostics from fixture files"
    );
    assert!(
        summary.parse_errors.is_empty(),
        "Did not expect parse errors in fixtures: {:?}",
        summary.parse_errors
    );
}

#[test]
fn test_check_project_diagnostics_are_sorted() {
    let summary = check_project(Path::new("tests/fixtures"));

    for window in summary.diagnostics.windows(2) {
        let a = &window[0];
        let b = &window[1];
        assert!(
            (a.file.as_str(), a.line, a.column) <= (b.file.as_str(), b.line, b.column),
            "Diagnostics not sorted: ({}, {}:{}) came before ({}, {}:{})",
            a.file, a.line, a.column, b.file, b.line, b.column
        );
    }
}

#[test]
fn test_check_project_single_file() {
    let summary = check_project(Path::new("tests/fixtures/yew_component.rs"));

    assert_eq!(summary.files_checked, 1);
    assert!(!summary.diagnostics.is_empty());
    assert!(
        summary.diagnostics.iter().all(|d| d.file.contains("yew")),
        "All diagnostics should come from the yew fixture"
    );
}

#[test]
fn test_check_project_filter_by_severity() {
    let summary = check_project(Path::new("tests/fixtures"));

    let errors: Vec<_> = summary
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    let warnings: Vec<_> = summary
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .collect();

    assert!(!errors.is_empty(), "Expected some errors in fixtures");
    assert!(!warnings.is_empty(), "Expected some warnings in fixtures");
    assert_eq!(errors.len() + warnings.len() + summary.diagnostics.iter().filter(|d| d.severity == Severity::Info).count(), summary.diagnostics.len());
}

#[test]
fn test_check_project_filter_by_rule() {
    let summary = check_project(Path::new("tests/fixtures"));

    let alt_text_issues: Vec<_> = summary
        .diagnostics
        .iter()
        .filter(|d| d.rule == Rule::AltText)
        .collect();

    assert!(
        !alt_text_issues.is_empty(),
        "Expected alt-text diagnostics in fixtures"
    );
}
