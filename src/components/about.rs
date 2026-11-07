use dioxus::prelude::*;

use crate::components::layout::Layout;

#[derive(Props, Clone, PartialEq)]
pub struct AboutProps {
    pub title: String,
    pub heading: String,
    pub description: String,
}

// About page rendered inside the shared Layout
#[component]
pub fn About(props: AboutProps) -> Element {
    rsx! {
        Layout { title: props.title.clone(),
            div { class: "mx-auto max-w-(--width-about) mt-10 px-4 md:px-0",
                h1 { class: "text-2xl md:text-3xl font-bold mb-4 tracking-wider text-text-primary",
                    "{props.heading}"
                }
                p { class: "leading-relaxed text-text bg-surface p-5 md:p-6 border border-border-thick rounded-lg",
                    "{props.description}"
                }
            }
        }
    }
}
