use leptos::leptos_dom::Transparent;
use leptos::{
    component, ev, view, Children, ChildrenFn, IntoView, RwSignal, SignalGet, SignalSet, View,
};

#[component]
pub fn Navbar(
    default_tab: &'static str,
    children: Children,
    active_tab: RwSignal<&'static str>,
) -> impl IntoView {
    active_tab.set(default_tab);
    let mut tabs_content = Vec::new();
    let mut button_bar = Vec::new();

    for child in children()
        .as_children()
        .iter()
        .filter_map(View::as_transparent)
        .filter_map(Transparent::downcast_ref::<TabView>)
        .cloned()
    {
        match child {
            TabView::Tab { name, children } => {
                tabs_content.push((name, children));
                button_bar.push(view! {<TabButton tab_name=name active_tab=active_tab />});
            }
            TabView::Button { children } => button_bar.push(children().into_view()),
        }
    }
    let active_tab_content = move || -> ChildrenFn {
        tabs_content
            .iter()
            .find(|(tab_name, _content)| tab_name == &active_tab.get())
            .expect("Tab not found")
            .1
            .clone()
    };

    view! {
        <div class="navbar">
            {button_bar}
        </div>
        {active_tab_content}
    }
}

#[component(transparent)]
pub fn Tab(name: &'static str, children: ChildrenFn) -> impl IntoView {
    TabView::Tab { name, children }
}

#[component(transparent)]
pub fn Button(children: ChildrenFn) -> impl IntoView {
    TabView::Button { children }
}

#[derive(Clone)]
enum TabView {
    Tab {
        name: &'static str,
        children: ChildrenFn,
    },
    Button {
        children: ChildrenFn,
    },
}

impl IntoView for TabView {
    fn into_view(self) -> View {
        Transparent::new(self).into_view()
    }
}

#[component]
fn TabButton(tab_name: &'static str, active_tab: RwSignal<&'static str>) -> impl IntoView {
    let button_click = move |_event: ev::MouseEvent| active_tab.set(tab_name);
    let button_class = move || match active_tab.get() == tab_name {
        true => "tab active",
        false => "tab",
    };
    view! {
        <button
            class=button_class
            on:click=button_click
        >
            {tab_name}
        </button>
    }
}
