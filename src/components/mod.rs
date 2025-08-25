#![allow(clippy::needless_pass_by_value)] // leptos has broken lifetime parsing

mod analysis;
mod app;
mod copy_to_clipboard;
mod dropdown;
mod footer;
mod navbar;
mod navigation;
mod program_window;
mod run_window;
mod state;
mod string_box;
mod toolbar;

pub use app::App;
