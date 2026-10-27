//! Which records have gone stale: not updated, and their consent not
//! confirmed, since a moment the caller names, the longest untouched first,
//! so the people a list has lost touch with are reviewed before they are
//! written to. A record that states no date is listed apart, never guessed.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::records::Human;
use crate::records::normalize::normalize_human;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaleRecord {
    pub id: String,
    pub display_name: Option<String>,
    /// The record's `updatedAt`, or its `createdAt` when it was never updated.
    pub last_touched: String,
    pub days_untouched: i64,
    /// When consent was last confirmed; `null` when the record never says.
    pub consent_updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Freshness {
    pub before: String,
    pub stale: Vec<StaleRecord>,
    /// Records that state neither `updatedAt` nor `createdAt`.
    pub undated: Vec<String>,
    pub fresh: usize,
}

fn moment(text: &str, id: &str, field: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(text)
        .with_context(|| format!("record {id} has a {field} that is not RFC 3339: {text}"))?
        .with_timezone(&Utc))
}

/// The records of `humans` untouched since `before`, judged at `at`.
pub fn stale_humans(humans: Vec<Human>, before: DateTime<Utc>, at: DateTime<Utc>) -> Result<Freshness> {
    let mut stale = Vec::new();
    let mut undated = Vec::new();
    let mut fresh = Vec::new();
    for input in humans {
        let human = normalize_human(input)?;
        let (field, text) = match (&human.updated_at, &human.created_at) {
            (Some(updated), _) => ("updatedAt", updated.clone()),
            (None, Some(created)) => ("createdAt", created.clone()),
            (None, None) => {
                undated.push(human.id);
                continue;
            }
        };
        let touched = moment(&text, &human.id, field)?;
        if touched >= before {
            fresh.push(human.id);
            continue;
        }
        stale.push(StaleRecord {
            days_untouched: (at - touched).num_days(),
            id: human.id,
            display_name: human.display_name,
            last_touched: text,
            consent_updated_at: human.consent.updated_at,
        });
    }
    stale.sort_by(|left, right| right.days_untouched.cmp(&left.days_untouched).then_with(|| left.id.cmp(&right.id)));
    Ok(Freshness { before: before.to_rfc3339(), stale, undated, fresh: fresh.len() })
}
