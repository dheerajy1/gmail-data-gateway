use std::{sync::LazyLock, time::Duration};

use chrono::{DateTime, FixedOffset};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::{task::JoinSet, time::sleep};

use super::model::IST_OFFSET_SECONDS;
use crate::{
    clients::gmail::GmailSession,
    errors::{api_error::ApiError, catalog::internal_server_error},
};

// Message ids requested per list page (SPEC section 5.3)
const PAGE_SIZE: usize = 100;

// Messages read at the same time, one group at a time (SPEC section 5.3)
const GROUP_SIZE: usize = 20;

// Matches every Unicode "other" character: controls, format characters, unassigned and private use (SPEC section 5.3), None only if the pattern failed to build
static CONTROL_CHARS: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r"\p{C}").ok());

// Pauses that spread the load on Gmail, tests set both to zero
#[derive(Debug, Clone, Copy)]
pub struct ReadPacing {
    // Pause before the next list page
    pub page_delay: Duration,

    // Pause between two groups of message reads
    pub group_delay: Duration,
}

impl ReadPacing {
    // Pauses of the old version: 300 ms between pages and 200 ms between groups
    pub const STANDARD: Self = Self {
        page_delay: Duration::from_millis(300),
        group_delay: Duration::from_millis(200),
    };
}

// One normalized mail, ready to go into a write-model event (SPEC section 5.3)
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MailRecord {
    // Gmail message id
    pub id: String,

    // Mail date in India time, like 2026-11-02T15:45:30+05:30
    pub date: String,

    // Cleaned From header
    pub from: String,

    // Cleaned Subject header
    pub subject: String,

    // Cleaned Gmail snippet
    pub snippet: String,
}

// Reply of the message list call
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListResponse {
    messages: Option<Vec<MessageRef>>,
    next_page_token: Option<String>,
}

// One entry of the message list
#[derive(Debug, Deserialize)]
struct MessageRef {
    id: Option<String>,
}

// Reply of the message get call in metadata format
#[derive(Debug, Deserialize)]
struct MessageResponse {
    id: Option<String>,
    snippet: Option<String>,
    payload: Option<Payload>,
}

// Payload of a message, only the headers are read
#[derive(Debug, Deserialize)]
struct Payload {
    headers: Option<Vec<MailHeader>>,
}

// One mail header
#[derive(Debug, Deserialize)]
struct MailHeader {
    #[serde(default)]
    name: String,

    #[serde(default)]
    value: String,
}

// One page of message ids and the token of the next page
struct Page {
    ids: Vec<String>,
    next: Option<String>,
}

// Removes control and invisible characters, collapses whitespace and trims (SPEC section 5.3)
fn clean_text(input: &str) -> String {
    let stripped = match CONTROL_CHARS.as_ref() {
        Some(pattern) => pattern.replace_all(input, "").into_owned(),
        None => input.chars().filter(|c| !c.is_control()).collect(),
    };

    stripped.split_whitespace().collect::<Vec<_>>().join(" ")
}

// Prepares a Date header for parsing: drops a trailing comment like (UTC) and the weekday, so a wrong weekday does not reject the date
fn normalise_date(text: &str) -> &str {
    let without_comment = text.split('(').next().unwrap_or(text).trim();

    without_comment
        .split_once(',')
        .map_or(without_comment, |(_, rest)| rest.trim())
}

// Turns a Date header into India time with the offset, None when it cannot be parsed (SPEC section 5.3)
fn format_ist_date(text: &str) -> Option<String> {
    let parsed = DateTime::parse_from_rfc2822(normalise_date(text)).ok()?;
    let ist = FixedOffset::east_opt(IST_OFFSET_SECONDS)?;

    Some(
        parsed
            .with_timezone(&ist)
            .format("%Y-%m-%dT%H:%M:%S%:z")
            .to_string(),
    )
}

// Lists one page of message ids, a Gmail failure is already an ApiError from the session
async fn list_page(
    session: &GmailSession,
    query: &str,
    page_token: Option<&str>,
) -> Result<Page, ApiError> {
    let max_results = PAGE_SIZE.to_string();
    let mut params = vec![("q", query), ("maxResults", max_results.as_str())];

    if let Some(token) = page_token {
        params.push(("pageToken", token));
    }

    let response = session.get("messages", &params).await?;

    // A reply that is not the expected json is an extraction failure
    let parsed: ListResponse = response.json().await.map_err(|error| {
        tracing::error!(target: "ExtractMails", "message list reply is not valid json: {error}");

        ApiError::from_entry(internal_server_error::EXTRACT_FAILED)
    })?;

    let ids = parsed
        .messages
        .unwrap_or_default()
        .into_iter()
        .filter_map(|message| message.id)
        .collect();

    Ok(Page {
        ids,
        next: parsed.next_page_token,
    })
}

// Reads one message in metadata format, None when it cannot be read or is incomplete (dropped silently, SPEC section 5.3)
async fn read_mail(session: &GmailSession, id: &str) -> Option<MailRecord> {
    let path = format!("messages/{id}");

    let response = session
        .get(
            &path,
            &[
                ("format", "metadata"),
                ("metadataHeaders", "Date"),
                ("metadataHeaders", "From"),
                ("metadataHeaders", "Subject"),
            ],
        )
        .await
        .ok()?;

    let message: MessageResponse = response.json().await.ok()?;

    let message_id = message.id?;
    let snippet = clean_text(message.snippet.as_deref().unwrap_or_default());
    let headers = message.payload?.headers?;

    let header = |name: &str| {
        headers
            .iter()
            .find(|header| header.name.eq_ignore_ascii_case(name))
            .map(|header| clean_text(&header.value))
            .unwrap_or_default()
    };

    let from = header("From");
    let subject = header("Subject");
    let date = format_ist_date(&header("Date"))?;

    // A mail without sender, subject or snippet is dropped
    if from.is_empty() || subject.is_empty() || snippet.is_empty() {
        return None;
    }

    Some(MailRecord {
        id: message_id,
        date,
        from,
        subject,
        snippet,
    })
}

// Reads a group of messages at the same time and keeps the order of the ids, unreadable ones are left out
async fn read_group(session: &'static GmailSession, ids: &[String]) -> Vec<MailRecord> {
    let mut tasks = JoinSet::new();

    for (index, id) in ids.iter().enumerate() {
        let id = id.clone();

        tasks.spawn(async move { (index, read_mail(session, &id).await) });
    }

    let mut results = Vec::with_capacity(ids.len());

    while let Some(joined) = tasks.join_next().await {
        // A task that panicked counts as an unreadable mail
        if let Ok((index, Some(mail))) = joined {
            results.push((index, mail));
        }
    }

    results.sort_by_key(|(index, _)| *index);

    results.into_iter().map(|(_, mail)| mail).collect()
}

// Reads the mails of a query newest first, page by page and group by group (SPEC section 5.3)
// With a limit it stops as soon as enough mails are read and cuts the list to the limit
pub async fn read_mails(
    session: &'static GmailSession,
    query: &str,
    limit: Option<u32>,
    pacing: ReadPacing,
) -> Result<Vec<MailRecord>, ApiError> {
    let max = limit.and_then(|limit| usize::try_from(limit).ok());

    let mut mails = Vec::new();
    let mut page_token: Option<String> = None;

    loop {
        let page = list_page(session, query, page_token.as_deref()).await?;

        if page.ids.is_empty() {
            break;
        }

        for (index, group) in page.ids.chunks(GROUP_SIZE).enumerate() {
            if index > 0 {
                sleep(pacing.group_delay).await;
            }

            mails.extend(read_group(session, group).await);

            if let Some(max) = max.filter(|max| mails.len() >= *max) {
                mails.truncate(max);

                return Ok(mails);
            }
        }

        match page.next {
            Some(token) => {
                page_token = Some(token);

                sleep(pacing.page_delay).await;
            }
            None => break,
        }
    }

    tracing::info!(target: "ExtractMails", "read {} mails", mails.len());

    Ok(mails)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The control character pattern builds, otherwise cleaning falls back to a smaller rule
    #[test]
    fn control_pattern_builds() {
        assert!(CONTROL_CHARS.is_some());
    }

    // Control and invisible characters are removed and whitespace is collapsed
    #[test]
    fn clean_text_removes_invisible_and_collapses_whitespace() {
        assert_eq!(clean_text("  Hello\u{200b}   world \n"), "Hello world");
        assert_eq!(clean_text("a\u{feff}b\u{202e}c"), "abc");
        assert_eq!(clean_text("tab\tseparated\u{a0}text"), "tabseparated text");
        assert_eq!(clean_text(""), "");
        assert_eq!(clean_text("   "), "");
    }

    // A numeric offset is converted to India time
    #[test]
    fn dates_are_converted_to_ist() {
        // 10:15:30 UTC is 15:45:30 in India
        assert_eq!(
            format_ist_date("Mon, 02 Nov 2026 10:15:30 +0000").as_deref(),
            Some("2026-11-02T15:45:30+05:30")
        );
        assert_eq!(
            format_ist_date("Mon, 2 Nov 2026 15:45:30 +0530").as_deref(),
            Some("2026-11-02T15:45:30+05:30")
        );
    }

    // A trailing comment and a wrong weekday do not reject the date
    #[test]
    fn dates_tolerate_comment_and_wrong_weekday() {
        assert_eq!(
            format_ist_date("Mon, 02 Nov 2026 10:15:30 +0000 (UTC)").as_deref(),
            Some("2026-11-02T15:45:30+05:30")
        );
        assert_eq!(
            format_ist_date("Fri, 02 Nov 2026 10:15:30 +0000").as_deref(),
            Some("2026-11-02T15:45:30+05:30")
        );
    }

    // A date that cannot be parsed gives None
    #[test]
    fn bad_dates_give_none() {
        assert_eq!(format_ist_date(""), None);
        assert_eq!(format_ist_date("not a date"), None);
        assert_eq!(format_ist_date("2026-11-02T10:15:30Z"), None);
    }
}
