# Changelog

All notable changes to this project are documented here. The format is based
on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
follows [Semantic Versioning](https://semver.org/) (pre-1.0: minor versions may
contain breaking changes).

## [0.2.0] - Unreleased

### Fixed

- **The CLI printed nothing when it found errors.** Output was buffered and the
  process exited without flushing, so error runs printed no (or truncated)
  diagnostics and `--out-file` wrote an empty file.
- `--only`/`--skip` rejected nothing: an unknown or misspelled rule name
  silently disabled linting. Unknown names are now a usage error.
- Files and macros that could not be parsed counted as clean. They are now
  reported with a line number and the run exits with status `2`.
- Dioxus `rsx!` (`div { class: "x" }`) is now actually parsed; previously every
  Dioxus file failed to parse.
- Markup macros nested in blocks, closures, attribute values and other macros
  (`vec![html! { … }]`, Leptos `fallback=|| view! { … }`) are now linted.
- Yew `{ for … }` and markup inside `if` blocks no longer break parsing.
- Many false positives, including errors for valid ARIA 1.2 roles (`generic`,
  `paragraph`, …), `autocomplete="work email"`, captioned media, labelled form
  controls, typed inputs (`<input type="checkbox" aria-checked>`), and
  `img-redundant-alt` substring matches.
- `anchor-ambiguous-text` now checks link content, not only `aria-label`.
- Columns are reported 1-based.
- Only `html!`/`html_nested!`/`view!`/`rsx!` bodies are parsed as markup, so
  ordinary macros such as `assert!(1 < 2)` no longer cause parse errors.
- Negative numbers such as `tabindex: -1` are read as static values.
- Root-level Yew `html! { if … { … } }` is linted.
- Misspelled `attr:aria-*` (Leptos) and `aria_*` (Dioxus) names are reported.
- Symbolic links (to files or directories) are not followed when collecting
  files, so nothing outside the scanned directory is linted.
- Accessible names follow the ARIA precedence: `aria-labelledby` over
  `aria-label` over content, and a link's descendants contribute their own
  name (`aria-label` over `alt`; `aria-hidden` content contributes nothing).
- A `<label>` only labels the control its `for` names, or else its first
  labelable descendant; buttons count as labelable, hidden inputs don't.
- `<form autocomplete>` only accepts `on` or `off`.
- A focusable `<hr role="separator">` must have `aria-valuenow`.
- A DPUB/Graphics role such as `doc-chapter` stops role fallback, so
  `role="doc-chapter heading"` is not checked as a heading.
- `disabled: false` and `multiple: false` (Dioxus, or `={false}`) count as
  unset.
- Ignore comments are found with a lexer: `//` inside strings no longer hides
  a directive, a directive inside a string no longer silences diagnostics, and
  block comments between a directive and its code are skipped.

### Added

- `// rsx-a11y-ignore[: rules] [-- reason]` and `// rsx-a11y-ignore-file`
  comments.
- Library API: `check_source`, `check_project_with`, `check_files`,
  `LintOptions`, `collect_rust_files`, `lints::run_lints`,
  `lints::lint_parsed`, and `LintSummary::{is_clean, errors, by_rule}`.
- `--list-rules` shows each rule's severity, description and a reference link;
  pretty output links each diagnostic to a WCAG reference.
- Exit status `2` for usage, I/O and parse errors; documented exit codes.
- Every element-level violation is reported, not only the first per rule.
- `anchor-is-valid` flags `javascript:` URLs and `<a onclick>` without `href`.
- CI on Linux, Windows and macOS with rustfmt, clippy, rustdoc, MSRV, wasm
  playground and packaging checks; end-to-end CLI tests and a diagnostics
  snapshot.

### Changed

- **Breaking:** `check_project` returns `io::Result<LintSummary>` and errors on
  a missing path.
- **Breaking:** `parse_source`/`parse_file` return a `ParsedSource` with
  elements, per-macro errors and ignore directives. `ParseError` is a struct
  with `kind`, `file`, `line` and `message`.
- **Breaking:** `--format json` writes one object with `diagnostics`,
  `parse_errors` and `summary` (including `files_with_parse_errors`), instead
  of a bare array.
- **Breaking:** `Tag`, `Role`, `Aria` and `Rule` implement `FromStr`/`Display`
  (replacing inherent `from_str`/`to_string`), are `Copy`, and are
  `#[non_exhaustive]`. `HtmlAttribute::value` is an `AttrValue` (with a new
  `Bare` variant) instead of an always-`Some` `Option`.
- **Breaking:** `HtmlElement::is_focusable` now includes `tabindex="-1"`; the
  new `is_tabbable` is the Tab-order check that `no-aria-hidden-on-focusable`
  and `interactive-supports-focus` use. Also new: `is_set` and `is_labelable`.
- `rayon` is behind a new `parallel` feature (enabled by `cli`); the unused
  `quote` and `walkdir` dependencies were removed.
- Minimum supported Rust version: 1.88.

## [0.1.4] - 2026-02-28

Initial public release.

[0.2.0]: https://github.com/CHildebrandt/rsx-a11y/compare/v0.1.4...v0.2.0
[0.1.4]: https://github.com/CHildebrandt/rsx-a11y/releases/tag/v0.1.4
