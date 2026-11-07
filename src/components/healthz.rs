use dioxus::prelude::*;

use crate::components::health_parts::{
    DbConnectionValue, DbStatusValue, Row, format_latency, format_number,
};
use crate::components::icons::CheckCircle;
use crate::components::layout::Layout;
use crate::features::v1::health::model::{AppHealthData, DbHealthData};

#[derive(Props, Clone, PartialEq)]
pub struct HealthzProps {
    pub title: String,
    pub app: AppHealthData,
    pub db: DbHealthData,
}

// Combined app and database health page rendered inside the shared Layout
#[component]
pub fn Healthz(props: HealthzProps) -> Element {
    let app = &props.app;
    let db = &props.db;

    rsx! {
        Layout { title: props.title.clone(),
            div { class: "max-w-(--width-health) mx-auto mt-8 space-y-6",
                h1 { class: "text-2xl font-bold text-text-primary", "Health Status" }

                div { class: "bg-surface border border-border-thick p-4 md:p-5 rounded-lg",
                    h2 { class: "text-lg font-semibold mb-3 text-text-primary", "Application" }
                    Row { label: "Status".to_string(),
                        CheckCircle { class: "text-success" }
                        "{app.status}"
                    }
                    Row { label: "Version".to_string(), "{app.version}" }
                    Row { label: "Uptime (ms)".to_string(), "{format_number(app.uptime_ms)}" }
                    Row { label: "Timestamp (UTC)".to_string(), "{app.timestamp_utc}" }
                }

                div { class: "bg-surface border border-border-thick p-4 md:p-5 rounded-lg",
                    h2 { class: "text-lg font-semibold mb-3 text-text-primary", "Database" }
                    Row { label: "Status".to_string(),
                        DbStatusValue { status: db.status }
                    }
                    Row { label: "Connection".to_string(),
                        DbConnectionValue { connection: db.database }
                    }
                    Row { label: "Latency (ms)".to_string(), "{format_latency(db.latency_ms)}" }
                }
            }
        }
    }
}
