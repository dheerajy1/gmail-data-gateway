use dioxus::prelude::*;

use crate::components::icons::{AlertTriangle, CheckCircle, XCircle};
use crate::features::v1::health::model::{DbConnection, DbStatus};

#[derive(Props, Clone, PartialEq)]
pub struct RowProps {
    pub label: String,
    pub children: Element,
}

// One label and value line inside a health card
#[component]
pub fn Row(props: RowProps) -> Element {
    rsx! {
        div { class: "flex justify-between py-2 border-b border-border last:border-0",
            span { class: "font-semibold text-text-muted", "{props.label}" }
            span { class: "font-mono text-sm flex items-center gap-2", {props.children} }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct DbStatusProps {
    pub status: DbStatus,
}

// Icon and text for the overall database status
#[component]
pub fn DbStatusValue(props: DbStatusProps) -> Element {
    match props.status {
        DbStatus::Up => rsx! {
            CheckCircle { class: "text-success" }
            "UP"
        },
        DbStatus::Down => rsx! {
            XCircle { class: "text-error" }
            "DOWN"
        },
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct DbConnectionProps {
    pub connection: DbConnection,
}

// Icon and text for the database connection state
#[component]
pub fn DbConnectionValue(props: DbConnectionProps) -> Element {
    match props.connection {
        DbConnection::Connected => rsx! {
            CheckCircle { class: "text-success" }
            "CONNECTED"
        },
        DbConnection::Connecting => rsx! {
            AlertTriangle { class: "text-warning" }
            "CONNECTING"
        },
        DbConnection::Disconnected => rsx! {
            XCircle { class: "text-error" }
            "DISCONNECTED"
        },
    }
}

// Formats a number with thousands separators, for example 12345 becomes 12,345
pub fn format_number(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

// Formats the optional latency, using a dash when it is missing
pub fn format_latency(value: Option<u64>) -> String {
    match value {
        Some(ms) => ms.to_string(),
        None => "—".to_string(),
    }
}
