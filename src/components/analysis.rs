use leptos::{component, view, IntoView, ReadSignal, Signal, SignalGet};
use std::str::FromStr;
use std::sync::Arc;

use crate::util;
use crate::util::Expression;

#[component]
pub fn Analysis(
    program: Signal<Option<Arc<Expression>>>,
    run_result: ReadSignal<Option<Result<String, String>>>,
) -> impl IntoView {
    let maybe_input = move || match (program.get(), run_result.get()) {
        (Some(program), Some(run_result)) => Some((program, run_result)),
        _ => None,
    };

    view! {
        {
            move || maybe_input().map(|(program, run_result)| view! {
                <div>
                    <AnalysisInner
                        expression=program
                        run_result=run_result/>
                </div>
            })
        }
    }
}

const MILLISECONDS_PER_WU: f64 = 0.5 / 1000.0;

#[component]
fn AnalysisInner(expression: Arc<Expression>, run_result: Result<String, String>) -> impl IntoView {
    let bounds = expression.bounds();
    // FIXME: Add conversion method to simplicity::Cost
    let milli_weight = u32::from_str(&bounds.cost.to_string()).unwrap();
    let weight = milli_weight.saturating_add(999) / 1000;
    let virtual_size = weight.div_ceil(4);
    let size = weight; // Simplicity programs are Taproot witness data
    let max_milliseconds = format!("{:.3}", f64::from(weight) * MILLISECONDS_PER_WU);
    let max_bytes = bounds.extra_cells.div_ceil(8);
    let compression = util::get_compression_factor(&expression);

    view! {
        <div class="analysis">
            <div class="flex analysis-header">
                <h2 class="analysis-title">Program Analysis</h2>
                <RunSuccess run_success=run_result.is_ok()/>
            </div>
            <div class="analysis-body">
                <div class="analysis-item">
                    <div class="analysis-item-label">Size:</div>
                    <div class="analysis-item-data">{size}B</div>
                </div>
                <div class="analysis-item">
                    <div class="analysis-item-label">Virtual size:</div>
                    <div class="analysis-item-data">{virtual_size}vB</div>
                </div>
                <div class="analysis-item">
                    <div class="analysis-item-label">Maximum memory:</div>
                    <div class="analysis-item-data">{max_bytes}B</div>
                </div>
                <div class="analysis-item">
                    <div class="analysis-item-label">Weight:</div>
                    <div class="analysis-item-data">{weight}WU</div>
                </div>
                <div class="analysis-item">
                    <div class="analysis-item-label">Maximum runtime:</div>
                    <div class="analysis-item-data">{max_milliseconds}ms</div>
                </div>
                <div class="analysis-item">
                    <div class="analysis-item-label">Program compression:</div>
                    <div class="analysis-item-data">{compression}x</div>
                </div>
            </div>

            <RunResultMessage run_result=run_result.clone()/>
        </div>
    }
}

#[component]
fn RunSuccess(run_success: bool) -> impl IntoView {
    match run_success {
        true => view! {
            <div class="program-status">
                <i class="fal fa-check-circle"></i>
                Program success
            </div>
        },
        false => view! {
            <div class="program-status is_error">
                <i class="fal fa-times-circle"></i>
                Program failure
            </div>
        },
    }
}

#[component]
fn RunResultMessage(run_result: Result<String, String>) -> impl IntoView {
    match run_result {
        Ok(_) => view! {
            <div></div>
        },
        Err(error) => {
            view! {
                <div class="program-status-error-message">
                    <pre>
                        {error}
                    </pre>
                </div>
            }
        }
    }
}
