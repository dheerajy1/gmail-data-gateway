use dioxus::prelude::*;

use crate::components::health_parts::{DbConnectionValue, DbStatusValue, Row, format_latency};
use crate::components::layout::Layout;
use crate::features::v1::health::model::DbHealthData;

#[derive(Props, Clone, PartialEq)]
pub struct HealthzDbProps {
    pub title: String,
    pub data: DbHealthData,
}

// Database health page rendered inside the shared Layout
#[component]
pub fn HealthzDb(props: HealthzDbProps) -> Element {
    let data = &props.data;

    rsx! {
        Layout { title: props.title.clone(),
            div { class: "max-w-(--width-health) mx-auto mt-8",
                h1 { class: "text-2xl font-bold mb-5 text-text-primary", "Database Health" }
                div { class: "bg-surface border border-border-thick p-4 md:p-5 rounded-lg",
                    Row { label: "Status".to_string(),
                        DbStatusValue { status: data.status }
                    }
                    Row { label: "Database".to_string(),
                        DbConnectionValue { connection: data.database }
                    }
                    Row { label: "Latency (ms)".to_string(), "{format_latency(data.latency_ms)}" }
                }
            }
        }
    }
}
