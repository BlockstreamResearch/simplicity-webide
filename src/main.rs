mod components;
mod examples;
mod function;
mod jet;
mod transaction;
mod util;

use components::App;
use leptos::{mount_to_body, view};
use leptos_router::Router;

#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! {
            <Router>
                <main>
                    <App/>
                </main>
            </Router>
        }
    });
}
