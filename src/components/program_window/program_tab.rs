use std::sync::Arc;

use itertools::Itertools;
use leptos::{
    component, create_effect, create_node_ref, create_rw_signal, ev, event_target_value, html,
    spawn_local, use_context, view, IntoView, RwSignal, Signal, SignalGet, SignalGetUntracked,
    SignalSet, SignalUpdate, SignalWith, SignalWithUntracked,
};
use simplicityhl::parse::ParseFromStr;
use simplicityhl::simplicity::jet::elements::ElementsEnv;
use simplicityhl::{elements, simplicity};
use simplicityhl::{CompiledProgram, SatisfiedProgram, WitnessValues};
use wasm_bindgen::JsCast;

use crate::components::copy_to_clipboard::CopyToClipboard;
use crate::function::Runner;

#[derive(Copy, Clone, Debug)]
pub struct Program {
    pub text: RwSignal<String>,
    cached_text: RwSignal<String>,
    pub lazy_cmr: RwSignal<Result<simplicity::Cmr, String>>,
    lazy_satisfied: RwSignal<Result<SatisfiedProgram, String>>,
}

impl Default for Program {
    fn default() -> Self {
        Self::new(String::default())
    }
}

impl Program {
    pub fn new(text: String) -> Self {
        let program = Self {
            text: create_rw_signal(text),
            cached_text: create_rw_signal(String::new()),
            lazy_cmr: create_rw_signal(Err(String::new())),
            lazy_satisfied: create_rw_signal(Err(String::new())),
        };
        program.update_on_read();
        program
    }

    pub fn is_empty(&self) -> bool {
        self.text.with_untracked(String::is_empty)
    }

    pub fn cmr(self) -> Result<simplicity::Cmr, String> {
        self.update_on_read();
        self.lazy_cmr.get_untracked()
    }

    pub fn satisfied(self) -> Result<SatisfiedProgram, String> {
        self.update_on_read();
        self.lazy_satisfied.get_untracked()
    }

    pub fn update_on_read(self) {
        let needs_update = self.text.with_untracked(|text| {
            self.cached_text
                .with_untracked(|cached_text| text != cached_text)
        });
        if !needs_update {
            return;
        }
        self.text.with_untracked(|text| {
            self.cached_text.set(text.clone());
            let compiled = simplicityhl::Arguments::parse_from_str(text)
                .map_err(|error| error.to_string())
                .and_then(|args| {
                    CompiledProgram::new(
                        text.as_str(),
                        args,
                        false, /* include debug symbols */
                    )
                });
            let cmr = compiled
                .as_ref()
                .map(|x| x.commit().cmr())
                .map_err(Clone::clone);
            self.lazy_cmr.set(cmr);
            let satisfied = compiled.and_then(|x| {
                let witness = WitnessValues::parse_from_str(text)?;
                x.satisfy(witness)
            });
            self.lazy_satisfied.set(satisfied);
        });
    }

    pub fn add_default_modules(self) {
        let (contains_witness, contains_param) = self
            .text
            .with_untracked(|text| (text.contains("mod witness"), text.contains("mod param")));
        if !contains_param {
            self.text
                .update(|text| text.insert_str(0, "mod param {}\n\n"));
        }
        if !contains_witness {
            self.text
                .update(|text| text.insert_str(0, "mod witness {}\n\n"));
        }
    }
}

#[derive(Copy, Clone)]
pub struct Runtime {
    program: Program,
    env: Signal<ElementsEnv<Arc<elements::Transaction>>>,
    pub run_succeeded: RwSignal<Option<bool>>,
    pub debug_output: RwSignal<String>,
    pub error_output: RwSignal<String>,
}

impl Runtime {
    pub fn new(program: Program, env: Signal<ElementsEnv<Arc<elements::Transaction>>>) -> Self {
        Self {
            program,
            env,
            run_succeeded: RwSignal::default(),
            debug_output: RwSignal::default(),
            error_output: RwSignal::default(),
        }
    }

    fn set_success(self, success: bool) {
        spawn_local(async move {
            self.run_succeeded.set(Some(success));
            gloo_timers::future::TimeoutFuture::new(500).await;
            self.run_succeeded.set(None);
        });
        web_sys::window()
            .as_ref()
            .map(web_sys::Window::navigator)
            .map(|navigator| match success {
                true => navigator.vibrate_with_duration(200),
                false => navigator.vibrate_with_duration(500),
            });
    }

    pub fn run(self) {
        self.debug_output.update(String::clear);
        let satisfied_program = match self.program.satisfied() {
            Ok(x) => x,
            Err(error) => {
                self.error_output.set(error);
                self.set_success(false);
                return;
            }
        };
        let mut runner = Runner::for_program(&satisfied_program);
        let success = self.env.with(|env| match runner.run(env) {
            Ok(..) => {
                self.error_output.update(String::clear);
                true
            }
            Err(error) => {
                self.error_output.set(error.to_string());
                false
            }
        });
        self.debug_output
            .set(runner.debug_output().into_iter().join("\n"));
        self.set_success(success);
    }
}

const TAB_KEY: u32 = 9;
const ENTER_KEY: u32 = 13;

#[component]
pub fn ProgramTab() -> impl IntoView {
    let program = use_context::<Program>().expect("program should exist in context");
    let runtime = use_context::<Runtime>().expect("runtime should exist in context");
    let textarea_ref = create_node_ref::<html::Textarea>();

    let update_program_text = move |event: ev::Event| {
        program.text.set(event_target_value(&event));
    };

    // Initialize CodeMirror when textarea is mounted (optional enhancement)
    create_effect(move |_prev_value| {
        if let Some(textarea) = textarea_ref.get() {
            // Only initialize once
            spawn_local(async move {
                // Small delay to ensure CodeMirror is loaded
                gloo_timers::future::TimeoutFuture::new(50).await;

                if let Some(window) = web_sys::window() {
                    // Check if CodeMirror and our init function exist
                    let has_codemirror =
                        js_sys::Reflect::has(&window, &"CodeMirror".into()).unwrap_or(false);

                    if has_codemirror {
                        if let Ok(simplicity_editor) =
                            js_sys::Reflect::get(&window, &"SimplicityEditor".into())
                        {
                            // Call SimplicityEditor.init() if it exists
                            if let Ok(init_fn) =
                                js_sys::Reflect::get(&simplicity_editor, &"init".into())
                            {
                                if let Some(init_fn) = init_fn.dyn_ref::<js_sys::Function>() {
                                    let textarea_id = "program-input";
                                    let initial_value = textarea.value();
                                    let _ = init_fn.call2(
                                        &simplicity_editor,
                                        &textarea_id.into(),
                                        &initial_value.into(),
                                    );
                                }
                            }
                        }
                    }
                }
            });
        }
    });

    // Sync CodeMirror when program.text changes (e.g. loading examples)
    create_effect(move |_| {
        let text = program.text.get();
        // Skip update if textarea already has this value (to avoid loops)
        if let Some(textarea) = textarea_ref.get() {
            if textarea.value() == text {
                return;
            }
        }

        spawn_local(async move {
            if let Some(window) = web_sys::window() {
                if let Ok(simplicity_editor) =
                    js_sys::Reflect::get(&window, &"SimplicityEditor".into())
                {
                    if let Ok(set_value_fn) =
                        js_sys::Reflect::get(&simplicity_editor, &"setValue".into())
                    {
                        if let Some(set_value_fn) = set_value_fn.dyn_ref::<js_sys::Function>() {
                            let _ = set_value_fn.call1(&simplicity_editor, &text.into());
                        }
                    }
                }
            }
        });
    });

    let insert_4_spaces = move || {
        let element = textarea_ref.get().expect("<textarea> should be mounted");
        if let Ok(Some(start)) = element.selection_start() {
            let start_ = start as usize; // safety: 32-bit machine of higher
            program.text.update(|s| s.insert_str(start_, "    "));
            let _result = element.set_selection_range(start + 4, start + 4);
        }
    };
    let delete_4_spaces = move || {
        let element = textarea_ref.get().expect("<textarea> should be mounted");
        if let Ok(Some(start)) = element.selection_start() {
            let start_ = start as usize; // safety: 32-bit machine of higher
            if start < 4 || program.text.with(|s| &s[start_ - 4..start_] != "    ") {
                return;
            }
            program
                .text
                .update(|s| s.replace_range(start_ - 4..start_, ""));
            let _result = element.set_selection_range(start - 4, start - 4);
        }
    };
    let handle_keydown = move |event: ev::KeyboardEvent| {
        if event.ctrl_key() && event.key_code() == ENTER_KEY {
            runtime.run();
        } else if event.key_code() == TAB_KEY {
            event.prevent_default();
            match event.shift_key() {
                false => insert_4_spaces(),
                true => delete_4_spaces(),
            }
        }
    };

    view! {
        <div class="tab-content">
            <div class="copy-program">
                <CopyToClipboard content=program.text class="copy-button" tooltip_below=true>
                    <i class="far fa-copy"></i>
                </CopyToClipboard>
            </div>
            <textarea
                class="program-input-field"
                placeholder="Enter your program here"
                rows="25"
                cols="80"
                spellcheck="false"
                prop:value=program.text
                on:input=update_program_text
                on:keydown=handle_keydown
                node_ref=textarea_ref
                name="program-input"
            >
                {program.text.get_untracked()}
            </textarea>
        </div>
    }
}
