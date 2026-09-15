//! Normalising one record: the fields it must carry, the shape of a date,
//! how an identity value compares, and the tags it may hold.

use anyhow::{Context, Result, bail};
use chrono::{DateTime, SecondsFormat, Utc};

use std::collections::{BTreeSet, HashSet};

use super::{Human, Identity};

pub(crate) fn required(value: &str, label: &str) -> Result<String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        bail!("{label} must be a non-empty string");
    }
    Ok(normalized)
}

pub(crate) fn optional(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        let normalized = item.split_whitespace().collect::<Vec<_>>().join(" ");
        (!normalized.is_empty()).then_some(normalized)
    })
}

pub(crate) fn normalize_date(value: Option<String>, label: &str) -> Result<Option<String>> {
    value
        .map(|item| {
            DateTime::parse_from_rfc3339(&item)
                .with_context(|| format!("{label} must be a valid ISO-8601 date"))
                .map(|date| {
                    date.with_timezone(&Utc)
                        .to_rfc3339_opts(SecondsFormat::Millis, true)
                })
        })
        .transpose()
}

pub(crate) fn normalize_identity_value(kind: &str, value: &str) -> String {
    match kind {
        "email" => value.trim().to_lowercase(),
        "phone" => {
            let has_plus = value.trim_start().starts_with('+');
            let digits: String = value.chars().filter(char::is_ascii_digit).collect();
            if has_plus {
                format!("+{digits}")
            } else {
                digits
            }
        }
        _ => value.trim().to_owned(),
    }
}

pub(crate) fn normalize_tags(tags: Vec<String>) -> Result<Vec<String>> {
    let mut result = BTreeSet::new();
    for tag in tags {
        result.insert(required(&tag, "tag")?.to_lowercase());
    }
    Ok(result.into_iter().collect())
}

pub fn normalize_human(mut human: Human) -> Result<Human> {
    human.id = required(&human.id, "human.id")?;
    human.display_name = optional(human.display_name);
    human.given_name = optional(human.given_name);
    human.family_name = optional(human.family_name);
    human.tags = normalize_tags(human.tags)?;
    human.created_at = normalize_date(human.created_at, "human.createdAt")?;
    human.updated_at = normalize_date(human.updated_at, "human.updatedAt")?;
    human.consent.updated_at = normalize_date(human.consent.updated_at, "human.consent.updatedAt")?;
    human.consent.source = optional(human.consent.source);

    let mut seen = HashSet::new();
    let mut identities = Vec::with_capacity(human.identities.len());
    for (index, identity) in human.identities.into_iter().enumerate() {
        let kind =
            required(&identity.kind, &format!("human.identities[{index}].type"))?.to_lowercase();
        let value = normalize_identity_value(
            &kind,
            &required(&identity.value, &format!("human.identities[{index}].value"))?,
        );
        if value.is_empty() {
            bail!("human.identities[{index}].value must be a non-empty string");
        }
        let key = format!("{kind}:{value}");
        if seen.insert(key) {
            identities.push(Identity {
                kind,
                value,
                verified: identity.verified,
            });
        } else if identity.verified
            && let Some(existing) = identities
                .iter_mut()
                .find(|item| item.kind == kind && item.value == value)
        {
            existing.verified = true;
        }
    }
    identities.sort_by(|left, right| {
        left.kind
            .cmp(&right.kind)
            .then_with(|| left.value.cmp(&right.value))
    });
    human.identities = identities;
    if human.display_name.is_none() {
        let composed = [human.given_name.as_deref(), human.family_name.as_deref()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
        human.display_name = (!composed.is_empty()).then_some(composed);
    }
    Ok(human)
}
