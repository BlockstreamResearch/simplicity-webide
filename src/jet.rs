use std::sync::Arc;

use simplicity::jet::Jet;
use simplicity::node::JetConstructible as _;
use simplicity::types::Context;
use simplicity::{BitMachine, ConstructNode, Value};
use simplicityhl::simplicity;

pub struct JetFailed;

/// Execute a jet on an input and inside an environment. Return the output.
pub fn execute_jet_with_env<J: Jet>(
    jet: &J,
    input: &Value,
    env: &J::Environment,
) -> Result<Value, JetFailed> {
    let prog = Context::with_context(|ctx| {
        Arc::<ConstructNode<J>>::jet(&ctx, *jet)
            .finalize_unpruned()
            .expect("a single jet definitely typechecks")
    });

    let mut mac = BitMachine::for_program(&prog).expect("a single jet is within limits");

    mac.input(input).expect("no problem with input");

    // The only execution error possible is a jet failure
    mac.exec(&prog, env).map_err(|_| JetFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[wasm_bindgen_test::wasm_bindgen_test]
    #[allow(dead_code)]
    fn wasm_sanity_checks() {
        assert!(simplicity::ffi::c_jets::sanity_checks());
    }
}
