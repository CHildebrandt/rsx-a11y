use leptos::prelude::*;
use rsx_a11y::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

/// Build the default editor text with the chosen macro wrapper.
fn default_code(macro_name: &str) -> String {
    format!(
        "fn component() {{\n    {}! {{\n        <img src=\"photo.jpg\" />\n        <div role=\"fake-role\" aria-hidden=\"yes\">\n            <div role=\"button\" onclick={{handler}}>{{\"Click me\"}}</div>\n            <marquee>{{\"Breaking news!\"}}</marquee>\n            <a href=\"#\">{{\"Link\"}}</a>\n            <h1></h1>\n        </div>\n    }}\n}}",
        macro_name
    )
}

/// Available macro wrappers.
const FRAMEWORKS: &[(&str, &str, &str)] = &[
    ("Yew", "html", "html!"),
    ("Leptos", "view", "view!"),
    ("Dioxus", "rsx", "rsx!"),
];

/// Run the linter on editor code (should be a complete Rust snippet).
fn lint_snippet(code: &str) -> LintResult {
    match rsx_a11y::parser::parse_source(code, "<playground>") {
        Ok(elements) => {
            let diagnostics: Vec<_> = rsx_a11y::lints::run_all_lints(&elements).collect();
            LintResult {
                diagnostics,
                elements_found: elements.len(),
                error: None,
            }
        }
        Err(e) => LintResult {
            diagnostics: vec![],
            elements_found: 0,
            error: Some(e.to_string()),
        },
    }
}

#[derive(Clone, PartialEq)]
struct LintResult {
    diagnostics: Vec<LintDiagnostic>,
    elements_found: usize,
    error: Option<String>,
}

/// Split text into plain-text and URL segments, rendering URLs as clickable `<a>` elements.
fn linkify(text: &str) -> Vec<leptos::tachys::view::any_view::AnyView> {
    let mut parts: Vec<leptos::tachys::view::any_view::AnyView> = Vec::new();
    let mut remaining = text;
    while let Some(start) = remaining
        .find("https://")
        .or_else(|| remaining.find("http://"))
    {
        if start > 0 {
            let before = remaining[..start].to_string();
            parts.push(before.into_any());
        }
        let after = &remaining[start..];
        let end = after
            .find(|c: char| c.is_whitespace())
            .unwrap_or(after.len());
        let url = after[..end].to_string();
        let url2 = url.clone();
        parts.push(
            view! { <a class="help-link" href={url} target="_blank" rel="noopener noreferrer">{url2}</a> }.into_any(),
        );
        remaining = &after[end..];
    }
    if !remaining.is_empty() {
        parts.push(remaining.to_string().into_any());
    }
    parts
}

#[component]
fn App() -> impl IntoView {
    let (code, set_code) = signal(default_code(FRAMEWORKS[0].1));
    let (framework_idx, set_framework_idx) = signal(0usize);

    let result = Memo::new(move |_| {
        let c = code.get();
        // framework_idx is read so the memo stays subscribed, but the
        // macro name is already embedded in the editor text.
        let _ = framework_idx.get();
        lint_snippet(&c)
    });

    view! {
        <div class="app">
            <header class="header">
                <img src="logo.png" alt="rsx-a11y logo" class="logo" />
                <div>
                    <h1>
                        "rsx-a11y Playground"
                    </h1>
                    <p class="subtitle">
                        "Paste HTML-like macro content to check for accessibility issues"
                    </p>
                </div>
            </header>

            <div class="toolbar">
                <label>"Framework: "</label>
                <div class="framework-tabs">
                    {FRAMEWORKS
                        .iter()
                        .enumerate()
                        .map(|(i, (name, _, label))| {
                            let name = *name;
                            let label = *label;
                            view! {
                                <button
                                    class:active=move || framework_idx.get() == i
                                    on:click=move |_| {
                                        let old_idx = framework_idx.get_untracked();
                                        if old_idx != i {
                                            let (_, old_macro, _) = FRAMEWORKS[old_idx];
                                            let (_, new_macro, _) = FRAMEWORKS[i];
                                            let old_prefix = format!("{}!", old_macro);
                                            let new_prefix = format!("{}!", new_macro);
                                            set_code.update(|c| {
                                                *c = c.replacen(&old_prefix, &new_prefix, 1);
                                            });
                                            set_framework_idx.set(i);
                                        }
                                    }
                                    title=label
                                >
                                    {name}
                                </button>
                            }
                        })
                        .collect::<Vec<_>>()}
                </div>
                <div class="stats">
                    {move || {
                        let r = result.get();
                        let diag_count = r.diagnostics.len();
                        let el_count = r.elements_found;
                        format!("{} element(s), {} issue(s)", el_count, diag_count)
                    }}
                </div>
            </div>

            <div class="editor-container">
                <div class="panel">
                    <div class="panel-header">"Input"</div>
                    <textarea
                        class="code-editor"
                        prop:value=move || code.get()
                        on:input=move |ev| {
                            let target = event_target::<web_sys::HtmlTextAreaElement>(&ev);
                            set_code.set(target.value());
                        }
                        spellcheck="false"
                        autocomplete="off"
                    ></textarea>
                </div>
                <div class="panel">
                    <div class="panel-header">"Results"</div>
                    <div class="results">
                        {move || {
                            let r = result.get();

                            if let Some(ref err) = r.error {
                                let err = err.clone();
                                return view! {
                                    <div class="parse-error">
                                        <strong>"Parse error: "</strong>
                                        {err}
                                    </div>
                                }
                                    .into_any();
                            }

                            if r.diagnostics.is_empty() {
                                return view! {
                                    <div class="no-issues">
                                        "✓ No accessibility issues found!"
                                    </div>
                                }
                                    .into_any();
                            }

                            view! {
                                <div class="diagnostics-list">
                                    {r
                                        .diagnostics
                                        .iter()
                                        .map(|d| {
                                            let severity_class = match d.severity {
                                                Severity::Error => "severity-error",
                                                Severity::Warning => "severity-warning",
                                                Severity::Info => "severity-info",
                                            };
                                            let severity_label = match d.severity {
                                                Severity::Error => "error",
                                                Severity::Warning => "warning",
                                                Severity::Info => "info",
                                            };
                                            let help = d.help.clone();
                                            let rule = d.rule.to_string();
                                            view! {
                                                <div class="diagnostic">
                                                    <div class="diag-header">
                                                        <span class={severity_class}>
                                                            {severity_label}
                                                        </span>
                                                        <span class="lint-id">{rule}</span>
                                                    </div>
                                                    <div class="diag-message">{d.message.clone()}</div>
                                                    <div class="diag-location">
                                                        {format!("line {}, col {}", d.line, d.column)}
                                                    </div>
                                                    {help
                                                        .map(|h| {
                                                            let fragments = linkify(&h);
                                                            view! {
                                                                <div class="diag-help">
                                                                    <span class="help-label">"help: "</span>
                                                                    {fragments}
                                                                </div>
                                                            }
                                                        })}
                                                </div>
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                </div>
                            }
                                .into_any()
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}
