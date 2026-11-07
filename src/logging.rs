use std::fmt;

use chrono::{FixedOffset, Offset, SecondsFormat, Utc};
use tracing::{Event, Subscriber};
use tracing_subscriber::{
    EnvFilter,
    fmt::{FmtContext, FormatEvent, FormatFields, format::Writer},
    registry::LookupSpan,
};

// Offset of India Standard Time from UTC, 5 hours 30 minutes
const IST_OFFSET_SECONDS: i32 = 5 * 3600 + 30 * 60;

// Current time in IST as an ISO string with milliseconds, like 2026-10-30T18:00:00.123+05:30
fn iso_now_ist() -> String {
    // The IST offset is a valid constant, UTC is only a fallback that cannot happen
    let ist = FixedOffset::east_opt(IST_OFFSET_SECONDS).unwrap_or_else(|| Utc.fix());

    Utc::now()
        .with_timezone(&ist)
        .to_rfc3339_opts(SecondsFormat::Millis, false)
}

// Formats every event as `<IST time>\t[Component:LEVEL]\t<message>`
struct LineFormat;

impl<S, N> FormatEvent<S, N> for LineFormat
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let meta = event.metadata();

        write!(
            writer,
            "{}\t[{}:{}]\t",
            iso_now_ist(),
            meta.target(),
            meta.level()
        )?;

        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

// Installs the global logger, RUST_LOG can change the level, the default is info
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .event_format(LineFormat)
        .init();
}
