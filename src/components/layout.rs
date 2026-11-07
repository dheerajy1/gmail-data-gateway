use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct LayoutProps {
    pub title: String,
    pub children: Element,
}

// Shared page shell: head tags, stylesheet link and body wrapper
#[component]
pub fn Layout(props: LayoutProps) -> Element {
    rsx! {
        head {
            meta { charset: "utf-8" }
            meta {
                name: "viewport",
                content: "width=device-width, initial-scale=1",
            }
            title { "{props.title}" }
            link { rel: "stylesheet", href: "/global.css" }
            link { rel: "icon", href: "/favicon.ico" }
        }
        body { class: "min-h-screen antialiased", {props.children} }
    }
}
