use leptos::{component, provide_context, view, IntoView, RwSignal};

use super::program_window::{select_example, Program, ProgramWindow, Runtime};
use crate::components::footer::Footer;
use crate::components::navigation::Navigation;
use crate::components::run_window::{HashCount, KeyCount, RunWindow, SignedData, TxEnv};
use crate::components::state::LocalStorage;
use crate::examples;
use crate::transaction::TxParams;
use crate::util::{HashedData, SigningKeys};

#[derive(Copy, Clone, Debug, Default)]
pub struct ActiveRunTab(pub RwSignal<&'static str>);

/// Controls which view is shown in `ProgramWindow`: "Run" (code editor) or "Analyze" (DAG view)
#[derive(Copy, Clone, Debug)]
pub struct ActiveProgramView(pub RwSignal<&'static str>);

impl Default for ActiveProgramView {
    fn default() -> Self {
        Self(RwSignal::new("Run"))
    }
}

#[component]
pub fn App() -> impl IntoView {
    let program = Program::load_from_storage().unwrap_or_default();
    provide_context(program);
    let tx_params = TxParams::load_from_storage().unwrap_or_default();
    let tx_env = TxEnv::new(program, tx_params);
    provide_context(tx_env);
    provide_context(SigningKeys::load_from_storage().unwrap_or_default());
    provide_context(SignedData::new(tx_env.lazy_env));
    provide_context(HashedData::load_from_storage().unwrap_or_default());
    provide_context(KeyCount::load_from_storage().unwrap_or_default());
    provide_context(HashCount::load_from_storage().unwrap_or_default());
    provide_context(Runtime::new(program, tx_env.lazy_env));
    provide_context(ActiveRunTab::default());
    provide_context(ActiveProgramView::default());

    if program.is_empty() {
        select_example(examples::get("✍️️ P2PK").expect("P2PK example should exist"));
    }

    view! {
        <Navigation />
        <section class="main-content">
            <ProgramWindow />
            <RunWindow />
        </section>
        <Footer />
    }
}
