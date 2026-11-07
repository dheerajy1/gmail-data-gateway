use dioxus::prelude::*;

use crate::components::health_parts::{Row, format_number};
use crate::components::icons::CheckCircle;
use crate::components::layout::Layout;
use crate::features::v1::health::model::AppHealthData;

#[derive(Props, Clone, PartialEq)]
pub struct HealthzAppProps {
    pub title: String,
    pub data: AppHealthData,
}

// App health page rendered inside the shared Layout
#[component]
pub fn HealthzApp(props: HealthzAppProps) -> Element {
    let data = &props.data;

    rsx! {
        Layout { title: props.title.clone(),
            div { class: "max-w-(--width-health) mx-auto mt-8",
                h1 { class: "text-2xl font-bold mb-5 text-text-primary", "App Health" }
                div { class: "bg-surface border border-border-thick p-4 md:p-5 rounded-lg",
                    Row { label: "Status".to_string(),
                        CheckCircle { class: "text-success" }
                        "{data.status}"
                    }
                    Row { label: "Version".to_string(), "{data.version}" }
                    Row { label: "Uptime (ms)".to_string(), "{format_number(data.uptime_ms)}" }
                    Row { label: "Timestamp (UTC)".to_string(), "{data.timestamp_utc}" }
                }
            }
        }
    }
}
