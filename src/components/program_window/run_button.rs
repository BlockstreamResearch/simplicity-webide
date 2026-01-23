use leptos::{component, ev, use_context, view, IntoView, SignalGet, SignalSet};

use crate::components::app::ActiveProgramView;
use crate::components::program_window::{Program, Runtime};
use crate::components::state::update_local_storage;

#[component]
pub fn RunButton() -> impl IntoView {
    let program = use_context::<Program>().expect("program should exist in context");
    let runtime = use_context::<Runtime>().expect("runtime should exist in context");
    let active_view = use_context::<ActiveProgramView>().expect("ActiveProgramView should exist");

    let run_program = move |_event: ev::MouseEvent| {
        // Switch back to code editor view
        active_view.0.set("Run");

        program.add_default_modules();
        update_local_storage();
        runtime.run();
    };
    let button_class = move || match runtime.run_succeeded.get() {
        None => "button run-button",
        Some(false) => "button run-button failure",
        Some(true) => "button run-button success",
    };

    view! {
        <button
            class=button_class
            on:click=run_program
        >
            <i class="fas fa-play"></i>
            " Run"
        </button>
    }
}
