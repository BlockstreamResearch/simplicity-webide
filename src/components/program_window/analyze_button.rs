use leptos::{component, use_context, view, IntoView, SignalGet, SignalSet};

use crate::components::app::ActiveProgramView;

#[component]
pub fn AnalyzeButton() -> impl IntoView {
    let active_view = use_context::<ActiveProgramView>().expect("ActiveProgramView should exist");

    let toggle_analyze = move |_| {
        let current = active_view.0.get();
        if current == "Analyze" {
            active_view.0.set("Run");
        } else {
            active_view.0.set("Analyze");
        }
    };

    let is_active = move || active_view.0.get() == "Analyze";

    view! {
        <button
            class="button"
            class:button-active=is_active
            on:click=toggle_analyze
        >
            " Analyze"
        </button>
    }
}
