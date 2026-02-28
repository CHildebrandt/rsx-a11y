<p align="center" dir="auto">
    <img src="https://raw.githubusercontent.com/CHildebrandt/rsx-a11y/main/assets/logo.png" width="200" alt="logo"/>
</p>

<h1 align="center">rsx-a11y</h1>

<p align="center">
    <a href="https://childebrandt.github.io/rsx-a11y/">Playground</a>
</p>

<p align="center">
    <a href="https://crates.io/crates/rsx-a11y"><img src="https://img.shields.io/crates/v/rsx-a11y.svg" alt="crates.io"/></a>
    <a href="https://docs.rs/rsx-a11y"><img src="https://docs.rs/rsx-a11y/badge.svg" alt="docs.rs"/></a>
    <a href="https://github.com/CHildebrandt/rsx-a11y/actions/workflows/build-and-test.yml"><img src="https://github.com/CHildebrandt/rsx-a11y/actions/workflows/build-and-test.yml/badge.svg" alt="build"/></a>
    <a href="https://github.com/CHildebrandt/rsx-a11y/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/rsx-a11y.svg" alt="license"/></a>
</p>

A static analysis tool that checks for ARIA and accessibility issues in Rust web framework code. Works with Rust web frameworks such as [Yew](https://yew.rs), [Leptos](https://leptos.dev) and [Dioxus](https://dioxuslabs.com) by parsing JSX-like macros `html!`, `view!`, and `rsx!` directly.

Inspired by [`eslint-plugin-jsx-a11y`](https://github.com/jsx-eslint/eslint-plugin-jsx-a11y) — the same idea, but for Rust.

## Quick Start

```sh
# Install the CLI
cargo install rsx-a11y

# Lint a file
rsx-a11y src/components/navbar.rs

# Lint an entire project
rsx-a11y src/

# JSON output (for CI)
rsx-a11y --format json src/
```

## Example Output

```
error: <img> element is missing an `alt` attribute. [alt-text]
  --> src/app.rs:13:13
  help: Add an `alt` attribute with descriptive text, or `alt=""` for decorative images.

warning: Redundant role "button" on <button>. This is the element's implicit role. [no-redundant-roles]
  --> src/app.rs:28:20
  help: Remove the `role` attribute.

warning: <div> with click handler must also have a keyboard event handler. [click-events-have-key-events]
  --> src/app.rs:37:13
  help: Add an `onkeydown` or `onkeyup` handler, or use an interactive element like <button> instead.

Checked 3 files in 12ms. Found 1 error, 2 warnings, 0 infos.
```

## Supported Frameworks

The tool parses all macro invocations using [`rstml`](https://github.com/rs-tml/rstml) — no compilation or framework dependencies required. Leptos-specific attribute prefixes like `on:click` and `class:active` are handled automatically.

## Lint Rules (36)

### Errors (10)

| Rule | Description |
|------|-------------|
| `alt-text` | Elements that require alt text (`<img>`, `<area>`, `<input type="image">`, `<object>`) must have it |
| `aria-props` | Unknown `aria-*` attribute (e.g. `aria-foo`) |
| `aria-proptypes` | Invalid value for a known ARIA attribute (e.g. `aria-hidden="yes"`) |
| `aria-role` | Unknown or abstract WAI-ARIA role (e.g. `role="banana"`, `role="widget"`) |
| `aria-unsupported-elements` | ARIA attributes on elements that don't support them (`<meta>`, `<script>`, etc.) |
| `autocomplete-valid` | Invalid `autocomplete` attribute value |
| `lang` | Invalid BCP 47 language tag |
| `no-aria-hidden-on-focusable` | `aria-hidden="true"` on a focusable element |
| `no-distracting-elements` | `<marquee>` or `<blink>` elements |
| `role-has-required-aria-props` | Missing required ARIA properties for a given role |

### Warnings (25)

| Rule | Description |
|------|-------------|
| `anchor-ambiguous-text` | `<a>` text must not be generic ("click here", "here", "link", "learn more") |
| `anchor-has-content` | `<a>` without discernible text content |
| `anchor-is-valid` | `<a>` with `href="#"`, empty `href`, or `javascript:void(0)` |
| `aria-activedescendant-has-tabindex` | Non-interactive element with `aria-activedescendant` needs `tabindex` |
| `click-events-have-key-events` | Click handler without corresponding keyboard handler on non-interactive element |
| `control-has-associated-label` | Interactive controls must have a text label |
| `heading-has-content` | Empty heading elements (`<h1>` through `<h6>`) |
| `html-has-lang` | `<html>` element without `lang` attribute |
| `iframe-has-title` | `<iframe>` without `title` attribute |
| `img-redundant-alt` | `<img>` alt text containing words like "image", "picture", "photo" |
| `interactive-supports-focus` | Element with interactive role and event handler must be focusable |
| `label-has-associated-control` | `<label>` without an associated form control |
| `media-has-caption` | `<video>` or `<audio>` without captions |
| `mouse-events-have-key-events` | `onmouseover`/`onmouseout` without `onfocus`/`onblur` |
| `no-access-key` | `accesskey` attribute (conflicts with screen readers) |
| `no-autofocus` | `autofocus` attribute (reduces usability) |
| `no-interactive-element-to-noninteractive-role` | Interactive element assigned a non-interactive role |
| `no-noninteractive-element-interactions` | Non-interactive element with event handlers |
| `no-noninteractive-element-to-interactive-role` | Non-interactive element assigned an interactive role |
| `no-noninteractive-tabindex` | `tabindex` on non-interactive elements |
| `no-redundant-roles` | Explicit role matches the element's implicit role |
| `no-static-element-interactions` | Static element (`<div>`, `<span>`) with event handlers but no role |
| `role-supports-aria-props` | ARIA property not supported by the element's role |
| `scope` | `scope` attribute on non-`<th>` elements |
| `tabindex-no-positive` | `tabindex` greater than 0 (unexpected tab order) |

### Info (1)

| Rule | Description |
|------|-------------|
| `prefer-tag-over-role` | Prefer semantic HTML element over ARIA role (e.g. `<button>` instead of `role="button"`) |

## CLI Options

```
rsx-a11y [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to a Rust file or directory to lint [default: .]

Options:
      --format <FORMAT>    Output format [default: pretty] [possible values: pretty, json]
  -q, --quiet              Only show errors (hide warnings and info)
      --list-rules         List all available lint rules and exit
      --only <RULES>       Only enable specific rules (comma-separated)
      --skip <RULES>       Disable specific rules (comma-separated)
      --out-file <PATH>    Write output to a file instead of stdout
  -h, --help               Print help
  -V, --version            Print version
```

### Examples

```sh
# Only check for alt text and ARIA role issues
rsx-a11y --only alt-text,aria-role src/

# Skip the autofocus warning
rsx-a11y --skip no-autofocus src/

# Errors only, for CI
rsx-a11y --quiet src/

# JSON for tooling integration
rsx-a11y --format json src/ > report.json

# Write output to a file
rsx-a11y --out-file report.txt src/
```

## How It Works

1. **Walk** — Finds all `.rs` files in the target path, processing them in parallel with [rayon](https://docs.rs/rayon). Skips `target/`, `node_modules/`, and hidden directories.
2. **Parse** — Uses [`syn`](https://docs.rs/syn) to parse each file's AST and visit all macro invocations. Uses [`rstml`](https://github.com/rs-tml/rstml) to parse the token stream inside each macro as HTML elements and attributes.
3. **Lint** — Runs all enabled lint rules against each extracted element. Each rule provides a severity, description, and help text with WCAG references.
4. **Report** — Outputs diagnostics sorted by file, line, and column. Supports colored terminal output and JSON.

Dynamic attribute values (e.g. `aria-hidden={is_hidden}`) are detected but skipped for value validation, since they can't be checked statically.

## Library / Testing API

You can use `rsx-a11y` as a library (without the CLI) to lint a project programmatically — useful for unit tests that enforce accessibility as part of CI.

Add it to your `Cargo.toml` with default features disabled:

```toml
[dev-dependencies]
rsx-a11y = { version = "*", default-features = false }
```

Then call `check_project` and assert on the returned `LintSummary`:

```rust
#[cfg(test)]
mod accessibility_tests {
    use std::path::Path;
    use rsx_a11y::prelude::*;

    #[test]
    fn no_accessibility_errors() {
        let summary = check_project(&Path::new(env!("CARGO_MANIFEST_DIR")));

        let errors: Vec<_> = summary.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .collect();

        assert!(errors.is_empty(), "accessibility errors found: {errors:#?}");
    }
}
```

`check_project` discovers `.rs` files, parses RSX macros, runs all 36 lint rules, and returns a `LintSummary` with sorted diagnostics — no filtering is applied, so you can filter by rule, severity, or file path after the fact:

```rust
#[cfg(test)]
mod accessibility_tests {
    use std::path::Path;
    use rsx_a11y::prelude::*;

    #[test]
    fn no_missing_alt_text() {
        let summary = check_project(&Path::new(env!("CARGO_MANIFEST_DIR")));

        let alt_issues: Vec<_> = summary.diagnostics
            .iter()
            .filter(|d| d.rule == Rule::AltText)
            .collect();

        assert!(alt_issues.is_empty(), "missing alt text: {alt_issues:#?}");
    }
}
```

## Playground

A browser-based playground is available in the `playground/` directory. Built with [Leptos](https://leptos.dev) and compiled to WebAssembly using [Trunk](https://trunkrs.dev), it lets you paste RSX snippets, select a framework (Yew, Leptos, or Dioxus), and see lint results in real time.

```sh
cd playground
trunk serve
```

## Development

```sh
# Run all tests
cargo test

# Run the linter on the test fixtures
cargo run -- tests/fixtures/

# List available rules
cargo run -- --list-rules
```

## References

- [WAI-ARIA 1.2 Specification](https://www.w3.org/TR/wai-aria-1.2/)
- [WAI-ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/)
- [eslint-plugin-jsx-a11y](https://github.com/jsx-eslint/eslint-plugin-jsx-a11y)

## License

MIT
