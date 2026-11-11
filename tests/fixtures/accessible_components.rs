//! Test fixture: accessible markup in Yew, Leptos and Dioxus.
//!
//! Every pattern here is correct, so linting this file must produce no
//! diagnostics at all. When a rule changes, add the correct form of the
//! pattern it checks here.

use dioxus::prelude::*;
use leptos::prelude::*;
use yew::prelude::*;

#[function_component]
fn YewSignup() -> Html {
    let items = vec!["a", "b"];
    let onclick = Callback::from(|_| ());
    let onkeydown = Callback::from(|_| ());

    html! {
        <html lang="en">
            <main>
                <h1>{"Create an account"}</h1>
                <img src="/logo.png" alt="Acme Inc." />
                <img src="/divider.png" alt="" />
                <form>
                    <label for="email">{"Email"}</label>
                    <input type="email" id="email" autocomplete="work email" />

                    <label>
                        {"Name"}
                        <input type="text" autocomplete="name" />
                    </label>

                    <input type="checkbox" id="terms" aria-describedby="terms-help" />
                    <label for="terms">{"I accept the terms"}</label>
                    <p id="terms-help">{"You can read them at any time."}</p>

                    <input type="hidden" name="csrf" value="token" />
                    <button type="submit">{"Sign up"}</button>
                </form>

                <ul>
                    { for items.iter().map(|item| html! { <li>{ item }</li> }) }
                </ul>

                <div role="switch" aria-checked="false" tabindex="0" {onclick} {onkeydown}>{"Dark mode"}</div>

                <a href="/privacy">{"Read our privacy policy"}</a>
                <iframe src="https://example.com/map" title="Office location map"></iframe>
                <video src="/intro.mp4">
                    <track kind="captions" src="/intro.vtt" srclang="en" />
                </video>
                <table>
                    <tr><th scope="col">{"Plan"}</th></tr>
                    <tr><td>{"Free"}</td></tr>
                </table>
            </main>
        </html>
    }
}

#[component]
fn LeptosTabs() -> impl IntoView {
    let (selected, set_selected) = signal(0);
    view! {
        <nav aria-label="Primary">
            <a href="/">"Home"</a>
            <a href="/about">"About us"</a>
        </nav>
        <ul role="tablist" aria-label="Settings">
            <li
                role="tab"
                aria-selected="true"
                tabindex="0"
                on:click=move |_| set_selected.set(0)
                on:keydown=move |_| set_selected.set(0)
            >
                "Profile"
            </li>
        </ul>
        <div role="tabpanel" tabindex="0" aria-label="Profile">
            <Show when=move || selected.get() == 0 fallback=|| view! { <p>"Nothing selected"</p> }>
                <h2>"Profile"</h2>
                <img src="/avatar.png" alt="Your avatar" />
            </Show>
        </div>
        <button aria-pressed="false" on:click=move |_| ()>"Mute"</button>
        <progress value="40" max="100" aria-label="Upload progress"></progress>
        <div aria-live="polite" aria-relevant="additions text"></div>
    }
}

#[component]
fn DioxusCard(title: String) -> Element {
    let items = vec![1, 2, 3];
    rsx! {
        article { aria_labelledby: "card-title",
            h2 { id: "card-title", "{title}" }
            img { src: "/card.png", alt: "A mountain lake at dawn" }
            for item in items.iter() {
                p { key: "{item}", "Item {item}" }
            }
            if items.is_empty() {
                p { "No items yet" }
            } else {
                button { r#type: "button", onclick: move |_| {}, "Clear" }
            }
            label { r#for: "search", "Search" }
            input { id: "search", r#type: "search" }
            a { href: "/cards", "Browse all cards" }
        }
    }
}
