use leptos::{component, view, IntoView, Signal, SignalGet};
use std::str::FromStr;
use std::sync::Arc;

use crate::util;
use crate::util::Expression;

#[component]
pub fn Analysis(program: Signal<Option<Arc<Expression>>>) -> impl IntoView {
    view! {
        {
            move || match program.get() {
                Some(expr) => view! {
                    <div>
                        <AnalysisInner expression=expr/>
                    </div>
                }.into_view(),
                None => view! {}.into_view(),
            }
        }
    }
}

const MILLISECONDS_PER_WU: f64 = 0.5 / 1000.0;

#[component]
fn AnalysisInner(expression: Arc<Expression>) -> impl IntoView {
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
            <div class="analysis-body">
                <div class="analysis-item">
                    <div class="analysis-item-label">"Size:"</div>
                    <div class="analysis-item-data">{size}"B"</div>
                </div>
                <div class="analysis-item">
                    <div class="analysis-item-label">"Virtual size:"</div>
                    <div class="analysis-item-data">{virtual_size}"vB"</div>
                </div>
                <div class="analysis-item">
                    <div class="analysis-item-label">"Maximum memory:"</div>
                    <div class="analysis-item-data">{max_bytes}"B"</div>
                </div>
                <div class="analysis-item">
                    <div class="analysis-item-label">"Weight:"</div>
                    <div class="analysis-item-data">{weight}"WU"</div>
                </div>
                <div class="analysis-item">
                    <div class="analysis-item-label">"Maximum runtime:"</div>
                    <div class="analysis-item-data">{max_milliseconds}"ms"</div>
                </div>
                <div class="analysis-item">
                    <div class="analysis-item-label">"Program compression:"</div>
                    <div class="analysis-item-data">{compression}"x"</div>
                </div>
            </div>
        </div>
    }
}
