use leptos::wasm_bindgen::JsValue;
use leptos::{component, create_rw_signal, ev, view, with, Children, IntoView, Signal, SignalSet};

fn try_write_clipboard(text: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let clipboard = window.navigator().clipboard();
    let js: &JsValue = clipboard.as_ref();
    if js.is_undefined() || js.is_null() {
        return;
    }
    let _ = clipboard.write_text(text);
}

#[component]
pub fn CopyToClipboard(
    #[prop(into)] content: Signal<String>,
    #[prop(into)] class: String,
    #[prop(default = false)] tooltip_below: bool,
    #[prop(optional)] on_copy: Option<Box<dyn Fn()>>,
    children: Children,
) -> impl IntoView {
    let tooltip_text = create_rw_signal("Copy");

    let button_click = move |_event: ev::MouseEvent| {
        with!(|content| try_write_clipboard(content));
        if let Some(cb) = &on_copy {
            cb();
        }
        tooltip_text.set("Copied!");
    };
    let button_mouseout = move |_event: ev::MouseEvent| {
        tooltip_text.set("Copy");
    };
    let tooltip_class = match tooltip_below {
        false => "tooltip-above",
        true => "tooltip-below",
    };

    view! {
        <div class=tooltip_class>
            <button
                class=class
                on:click=button_click
                on:mouseout=button_mouseout
            >
                <span class="tooltip-text">{tooltip_text}</span>
                {children()}
            </button>
        </div>
    }
}
