mod address_button;
mod analyze_button;
mod analyze_view;
mod examples_dropdown;
mod help_button;
mod program_tab;
mod run_button;
mod share_button;
mod tools_dropdown;
mod transaction_button;

use leptos::create_signal;
use leptos::{component, use_context, view, IntoView, SignalGet, SignalSet};

use self::address_button::AddressButton;
use self::analyze_button::AnalyzeButton;
use self::analyze_view::AnalyzeView;
use self::examples_dropdown::ExamplesDropdown;
use self::help_button::HelpButton;
use self::program_tab::ProgramTab;
use self::run_button::RunButton;
use self::share_button::ShareButton;
use self::transaction_button::TransactionButton;
use crate::components::app::ActiveProgramView;
use crate::components::toolbar::Toolbar;

pub use self::examples_dropdown::select_example;
pub use self::program_tab::{Program, Runtime};

#[component]
pub fn ProgramWindow() -> impl IntoView {
    let (mobile_open, set_mobile_open) = create_signal(false);
    let active_view = use_context::<ActiveProgramView>().expect("ActiveProgramView should exist");

    view! {
        <Toolbar>
            <RunButton />
            <ExamplesDropdown />

            <div class="mobile-hidden"  class:open = move || mobile_open.get() >
                <AddressButton />
                <TransactionButton />
                <AnalyzeButton />
                <ShareButton />
                <HelpButton />
            </div>

            <div
                class="hamburger-container"
                class:hamburger-active=mobile_open
                on:click=move |_| set_mobile_open.set(!mobile_open.get())
            >
                {move || if mobile_open.get() {
                    view! { <i class="fa-solid fa-xmark"></i> }
                } else {
                    view! { <i class="fa-solid fa-bars"></i>}
                }}
            </div>
        </Toolbar>

        // Toggle between code editor and analyze view
        {move || match active_view.0.get() {
            "Analyze" => view! { <AnalyzeView /> }.into_view(),
            _ => view! { <ProgramTab /> }.into_view(),
        }}
    }
}
