use leptos::{
    component, create_effect, create_memo, create_rw_signal, use_context, view, wasm_bindgen,
    IntoView, RwSignal, Signal, SignalGet, SignalSet,
};
use std::sync::Arc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};

use crate::components::analysis::Analysis;
use crate::components::program_window::Runtime;
use crate::util::dag::{build_dag_export, NodeMeta};
use crate::util::Expression;

#[component]
pub fn AnalyzeView() -> impl IntoView {
    let runtime = use_context::<Runtime>().expect("Runtime should exist in context");

    // Get program expression signal
    let program_expr: Signal<Option<Arc<Expression>>> =
        Signal::derive(move || runtime.program_expr().get());

    // Selected node id for details panel
    let selected: RwSignal<Option<String>> = create_rw_signal(None);

    // Build DAG export when program exists
    let dag_export = create_memo(move |_| program_expr.get().map(|expr| build_dag_export(&expr)));

    // Build node lookup map
    let node_map = create_memo(move |_| {
        dag_export.get().map(|dag| {
            dag.nodes
                .iter()
                .map(|n| (n.id.clone(), n.clone()))
                .collect::<std::collections::HashMap<String, NodeMeta>>()
        })
    });

    // Get selected node details
    let selected_node = move || {
        let sel_id = selected.get()?;
        let map = node_map.get()?;
        map.get(&sel_id).cloned()
    };

    view! {
        <div class="tab-content analyze-view">
            // Existing Analysis metrics
            <Analysis program=program_expr />

            // DAG Visualization section
            <div class="dag-section">
                <h3 class="dag-section-title">"Committed DAG"</h3>

                {move || match dag_export.get() {
                    Some(dag) => view! {
                        <div class="dag-layout">
                            <DagCanvas dag_json=serde_json::to_string(&dag).unwrap_or_default() selected=selected />
                            <DagDetails node=selected_node() />
                        </div>
                    }.into_view(),
                    None => view! {
                        <div class="dag-empty-state">
                            <i class="fas fa-project-diagram"></i>
                            <p>"Run your program to visualize the DAG"</p>
                        </div>
                    }.into_view(),
                }}
            </div>
        </div>
    }
}

#[component]
fn DagCanvas(dag_json: String, selected: RwSignal<Option<String>>) -> impl IntoView {
    let container_id = "simplicity-dag-container";

    // Set up click handler and render
    create_effect(move |_| {
        // Set up the click handler on window
        let window = web_sys::window().expect("no window");

        // Create click callback
        let selected_clone = selected;
        let cb = Closure::wrap(Box::new(move |node_id: String| {
            selected_clone.set(Some(node_id));
        }) as Box<dyn Fn(String)>);

        // Store callback on window for JS to call
        let _ = js_sys::Reflect::set(
            &window,
            &JsValue::from_str("dagNodeClickHandler"),
            cb.as_ref(),
        );
        cb.forget();

        // Call the global renderDag function
        let render_fn = js_sys::Reflect::get(&window, &JsValue::from_str("renderDag")).ok();
        if let Some(func) = render_fn {
            if let Ok(func) = func.dyn_into::<js_sys::Function>() {
                let _ = func.call2(
                    &JsValue::NULL,
                    &JsValue::from_str(container_id),
                    &JsValue::from_str(&dag_json),
                );
            }
        }
    });

    view! {
        <div class="dag-canvas-container">
            <div id=container_id class="dag-canvas"></div>
            <div class="dag-controls">
                <button class="dag-control-btn" id="dag-zoom-in" title="Zoom In">
                    <i class="fas fa-plus"></i>
                </button>
                <button class="dag-control-btn" id="dag-zoom-out" title="Zoom Out">
                    <i class="fas fa-minus"></i>
                </button>
                <button class="dag-control-btn" id="dag-zoom-reset" title="Reset View">
                    <i class="fas fa-compress-arrows-alt"></i>
                </button>
            </div>
        </div>
    }
}

#[component]
fn DagDetails(node: Option<NodeMeta>) -> impl IntoView {
    view! {
        <div class="dag-details">
            {match node {
                Some(n) => view! {
                    <div class="dag-details-content">
                        <div class="dag-detail-item">
                            <span class="dag-detail-label">"Kind:"</span>
                            <span class="dag-detail-value dag-kind" data-kind-class=n.kind_class.clone()>
                                {n.kind.clone()}
                            </span>
                        </div>
                        <div class="dag-detail-item">
                            <span class="dag-detail-label">"Type:"</span>
                            <span class="dag-detail-value dag-type">{n.type_arrow.clone()}</span>
                        </div>
                        <div class="dag-detail-item">
                            <span class="dag-detail-label">"Description:"</span>
                            <span class="dag-detail-value">{n.desc.clone()}</span>
                        </div>
                        <div class="dag-detail-item dag-detail-cmr">
                            <span class="dag-detail-label">"CMR:"</span>
                            <code class="dag-detail-value dag-cmr">{n.cmr.clone()}</code>
                        </div>
                        {n.extra.clone().map(|extra| view! {
                            <div class="dag-detail-item">
                                <span class="dag-detail-label">"Extra:"</span>
                                <code class="dag-detail-value dag-extra">{extra}</code>
                            </div>
                        })}
                    </div>
                }.into_view(),
                None => view! {
                    <div class="dag-details-empty">
                        <i class="fas fa-mouse-pointer"></i>
                        <p>"Click a node to see details"</p>
                    </div>
                }.into_view(),
            }}
        </div>
    }
}
