use leptos::{component, use_context, view, IntoView, SignalWithUntracked};

use crate::components::copy_to_clipboard::CopyToClipboard;
use crate::components::program_window::Program;
use crate::url_sharing;

#[component]
pub fn ShareButton() -> impl IntoView {
    let program = use_context::<Program>().expect("program should exist in context");

    let share_url = move || {
        program.text.with_untracked(|text| {
            url_sharing::build_share_url(text).unwrap_or_else(|| "Empty program".to_string())
        })
    };
    let update_hash = Box::new(move || {
        program.text.with_untracked(|text| {
            url_sharing::set_url_hash(text);
        });
    });

    view! {
        <CopyToClipboard content=share_url on_copy=update_hash class="button" tooltip_below=true>
            " Share"
        </CopyToClipboard>
    }
}
