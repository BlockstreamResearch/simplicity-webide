use js_sys::Date;
use leptos::{component, use_context, view, IntoView, SignalWith};

use crate::components::program_window::Runtime;
use crate::components::string_box::{ErrorBox, NeutralBox, SuccessBox};

#[component]
pub fn ExecutionTab() -> impl IntoView {
    let runtime = use_context::<Runtime>().expect("runtime should exist in context");
    let success_string = move || {
        runtime.error_output.with(|error| match error.is_empty() {
            true => format!("{}: Success.", get_local_datetime()),
            false => String::new(),
        })
    };
    let failure_string = move || {
        runtime.error_output.with(|error| match error.is_empty() {
            true => String::new(),
            false => format!("{}:\n{error}", get_local_datetime()),
        })
    };

    view! {
        <div class="tab-content">
            <SuccessBox success=success_string />
            <ErrorBox error=failure_string />
            <NeutralBox neutral=runtime.debug_output />
        </div>
    }
}

fn get_local_datetime() -> String {
    let date = Date::new_0();
    date.to_iso_string().as_string().unwrap()
}
