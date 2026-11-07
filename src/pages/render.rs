use axum::response::Html;
use dioxus::prelude::*;

// Renders a component without props into a full html document
pub fn render(app: fn() -> Element) -> Html<String> {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    wrap(dioxus_ssr::render(&dom))
}

// Renders a component with props into a full html document
pub fn render_with_props<P: Clone + 'static>(app: fn(P) -> Element, props: P) -> Html<String> {
    let mut dom = VirtualDom::new_with_props(app, props);
    dom.rebuild_in_place();
    wrap(dioxus_ssr::render(&dom))
}

// Adds the doctype and html tag around the rendered head and body
fn wrap(body: String) -> Html<String> {
    Html(format!("<!DOCTYPE html><html lang=\"en\">{body}</html>"))
}
