use leptos::{component, view, IntoView};

#[component]
pub fn HelpButton() -> impl IntoView {
    view! {
        <form action="https://github.com/uncomputable/simfony-webide/blob/master/doc/README.md" target="_blank">
            <button class="button" type="submit">
                " Help"
            </button>
        </form>
    }
}
