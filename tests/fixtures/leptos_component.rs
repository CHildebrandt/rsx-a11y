//! Test fixture: Leptos component with various accessibility issues.

use leptos::*;

#[component]
fn BadLeptosComponent() -> impl IntoView {
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|c| *c += 1);

    view! {
        <div>
            // missing-alt-text
            <img src="photo.jpg" />

            // invalid-aria-attribute
            <div aria-roledescriptions="test">{"Content"}</div>

            // invalid-aria-value
            <button aria-pressed="yes">{"Toggle"}</button>

            // invalid-role
            <div role="superbutton">{"Not a button"}</div>

            // redundant-role on nav
            <nav role="navigation">{"Nav links"}</nav>

            // click without keyboard on non-interactive element
            <div on:click=on_click>{"Click me"}</div>

            // anchor with empty href
            <a href="">{"Empty link"}</a>

            // video without caption
            <video src="video.mp4"></video>
        </div>
    }
}

#[component]
fn GoodLeptosComponent() -> impl IntoView {
    let (count, set_count) = create_signal(0);

    view! {
        <div>
            <img src="photo.jpg" alt="A beautiful sunset over the ocean" />

            <div aria-label="Main content" role="main">
                {"Content"}
            </div>

            <button on:click=move |_| set_count.update(|c| *c += 1)>
                {"Count: "} {count}
            </button>

            <nav aria-label="Primary navigation">
                <a href="/home">{"Home"}</a>
                <a href="/about">{"About"}</a>
            </nav>

            <input aria-label="Search" />

            <video src="video.mp4" aria-label="Tutorial video"></video>
        </div>
    }
}
