use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct IconProps {
    #[props(default = String::new())]
    pub class: String,
}

// Green check inside a circle, used for healthy states
#[component]
pub fn CheckCircle(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "16",
            height: "16",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            path { d: "m9 12 2 2 4-4" }
        }
    }
}

// Cross inside a circle, used for failed states
#[component]
pub fn XCircle(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "16",
            height: "16",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            path { d: "m15 9-6 6" }
            path { d: "m9 9 6 6" }
        }
    }
}

// Warning triangle, used for in-between states
#[component]
pub fn AlertTriangle(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "16",
            height: "16",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3" }
            path { d: "M12 9v4" }
            path { d: "M12 17h.01" }
        }
    }
}

// Person outline, used inside the username input
#[component]
pub fn User(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "16",
            height: "16",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2" }
            circle { cx: "12", cy: "7", r: "4" }
        }
    }
}

// Padlock, used inside the password input
#[component]
pub fn Lock(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "{props.class}",
            width: "16",
            height: "16",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect {
                width: "18",
                height: "11",
                x: "3",
                y: "11",
                rx: "2",
                ry: "2",
            }
            path { d: "M7 11V7a5 5 0 0 1 10 0v4" }
        }
    }
}
