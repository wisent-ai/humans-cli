use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    #[serde(rename = "type")]
    pub kind: String,
    pub value: String,
    #[serde(default)]
    pub verified: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Consent {
    #[serde(default)]
    pub email: bool,
    #[serde(default)]
    pub sms: bool,
    #[serde(default)]
    pub profiling: bool,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Human {
    pub id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub given_name: Option<String>,
    #[serde(default)]
    pub family_name: Option<String>,
    #[serde(default)]
    pub identities: Vec<Identity>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub attributes: BTreeMap<String, Value>,
    #[serde(default)]
    pub consent: Consent,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroup {
    pub key: String,
    pub identity_type: String,
    pub identity_value: String,
    pub human_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    #[serde(default)]
    pub all_tags: Vec<String>,
    #[serde(default)]
    pub any_tags: Vec<String>,
    #[serde(default)]
    pub identity_types: Vec<String>,
    #[serde(default)]
    pub require_verified_identity: bool,
    #[serde(default)]
    pub consent: BTreeMap<String, bool>,
    #[serde(default)]
    pub attributes: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeResult {
    pub human: Human,
    pub source_ids: Vec<String>,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanSummary {
    pub humans: usize,
    pub unique_identities: usize,
    pub verified_identities: usize,
    pub duplicate_groups: usize,
    pub identity_types: Vec<CountEntry>,
    pub tags: Vec<CountEntry>,
    pub consent: ConsentSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct CountEntry {
    pub key: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConsentSummary {
    pub email: usize,
    pub sms: usize,
    pub profiling: usize,
}

fn required(value: &str, label: &str) -> Result<String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        bail!("{label} must be a non-empty string");
    }
    Ok(normalized)
}

fn optional(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        let normalized = item.split_whitespace().collect::<Vec<_>>().join(" ");
        (!normalized.is_empty()).then_some(normalized)
    })
}

fn normalize_date(value: Option<String>, label: &str) -> Result<Option<String>> {
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

fn normalize_identity_value(kind: &str, value: &str) -> String {
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

fn normalize_tags(tags: Vec<String>) -> Result<Vec<String>> {
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

pub fn find_duplicates(inputs: Vec<Human>) -> Result<Vec<DuplicateGroup>> {
    let mut identities: HashMap<String, (String, String, BTreeSet<String>)> = HashMap::new();
    for input in inputs {
        let human = normalize_human(input)?;
        for identity in human.identities {
            let key = format!("{}:{}", identity.kind, identity.value);
            identities
                .entry(key)
                .or_insert_with(|| (identity.kind, identity.value, BTreeSet::new()))
                .2
                .insert(human.id.clone());
        }
    }
    let mut duplicates: Vec<_> = identities
        .into_iter()
        .filter_map(|(key, (identity_type, identity_value, ids))| {
            (ids.len() > 1).then(|| DuplicateGroup {
                key,
                identity_type,
                identity_value,
                human_ids: ids.into_iter().collect(),
            })
        })
        .collect();
    duplicates.sort_by(|left, right| left.key.cmp(&right.key));
    Ok(duplicates)
}

fn consent_matches(consent: &Consent, required: &BTreeMap<String, bool>) -> bool {
    required.iter().all(|(key, expected)| {
        let actual = match key.as_str() {
            "email" => Some(consent.email),
            "sms" => Some(consent.sms),
            "profiling" => Some(consent.profiling),
            _ => None,
        };
        actual == Some(*expected)
    })
}

pub fn matches_segment(human: &Human, segment: &Segment) -> bool {
    let all_tags = segment.all_tags.iter().map(|tag| tag.to_lowercase());
    let any_tags: Vec<_> = segment
        .any_tags
        .iter()
        .map(|tag| tag.to_lowercase())
        .collect();
    let identity_types: Vec<_> = segment
        .identity_types
        .iter()
        .map(|kind| kind.to_lowercase())
        .collect();
    all_tags.into_iter().all(|tag| human.tags.contains(&tag))
        && (any_tags.is_empty() || any_tags.iter().any(|tag| human.tags.contains(tag)))
        && (identity_types.is_empty()
            || identity_types.iter().any(|kind| {
                human.identities.iter().any(|identity| {
                    identity.kind == *kind
                        && (!segment.require_verified_identity || identity.verified)
                })
            }))
        && consent_matches(&human.consent, &segment.consent)
        && segment
            .attributes
            .iter()
            .all(|(key, expected)| human.attributes.get(key) == Some(expected))
}

pub fn segment_humans(inputs: Vec<Human>, segment: Segment) -> Result<Vec<Human>> {
    let mut humans = Vec::new();
    for input in inputs {
        let human = normalize_human(input)?;
        if matches_segment(&human, &segment) {
            humans.push(human);
        }
    }
    humans.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(humans)
}

fn merge_optional(
    field: &str,
    target: &mut Option<String>,
    incoming: Option<String>,
    conflicts: &mut BTreeSet<String>,
) {
    match (target.as_ref(), incoming) {
        (None, value) => *target = value,
        (Some(current), Some(value)) if current != &value => {
            conflicts.insert(field.to_owned());
        }
        _ => {}
    }
}

pub fn merge_humans(inputs: Vec<Human>, target_id: Option<String>) -> Result<MergeResult> {
    if inputs.is_empty() {
        bail!("humans must be a non-empty array");
    }
    let mut normalized = inputs
        .into_iter()
        .map(normalize_human)
        .collect::<Result<Vec<_>>>()?;
    let mut merged = normalized.remove(0);
    let mut source_ids = vec![merged.id.clone()];
    if let Some(target_id) = target_id {
        merged.id = required(&target_id, "targetId")?;
    }
    let mut conflicts = BTreeSet::new();
    let mut identity_map: BTreeMap<(String, String), bool> = merged
        .identities
        .iter()
        .map(|identity| {
            (
                (identity.kind.clone(), identity.value.clone()),
                identity.verified,
            )
        })
        .collect();
    let mut tags: BTreeSet<String> = merged.tags.iter().cloned().collect();

    for human in normalized {
        source_ids.push(human.id);
        merge_optional(
            "displayName",
            &mut merged.display_name,
            human.display_name,
            &mut conflicts,
        );
        merge_optional(
            "givenName",
            &mut merged.given_name,
            human.given_name,
            &mut conflicts,
        );
        merge_optional(
            "familyName",
            &mut merged.family_name,
            human.family_name,
            &mut conflicts,
        );
        merge_optional(
            "createdAt",
            &mut merged.created_at,
            human.created_at,
            &mut conflicts,
        );
        merge_optional(
            "updatedAt",
            &mut merged.updated_at,
            human.updated_at,
            &mut conflicts,
        );
        for identity in human.identities {
            identity_map
                .entry((identity.kind, identity.value))
                .and_modify(|verified| *verified |= identity.verified)
                .or_insert(identity.verified);
        }
        tags.extend(human.tags);
        for (key, value) in human.attributes {
            match merged.attributes.get(&key) {
                None => {
                    merged.attributes.insert(key, value);
                }
                Some(existing) if existing != &value => {
                    conflicts.insert(format!("attributes.{key}"));
                }
                _ => {}
            }
        }
        merged.consent.email |= human.consent.email;
        merged.consent.sms |= human.consent.sms;
        merged.consent.profiling |= human.consent.profiling;
    }
    merged.identities = identity_map
        .into_iter()
        .map(|((kind, value), verified)| Identity {
            kind,
            value,
            verified,
        })
        .collect();
    merged.tags = tags.into_iter().collect();
    source_ids.sort();
    source_ids.dedup();
    Ok(MergeResult {
        human: normalize_human(merged)?,
        source_ids,
        conflicts: conflicts.into_iter().collect(),
    })
}

fn count_entries(counts: BTreeMap<String, usize>) -> Vec<CountEntry> {
    let mut result: Vec<_> = counts
        .into_iter()
        .map(|(key, count)| CountEntry { key, count })
        .collect();
    result.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.key.cmp(&right.key))
    });
    result
}

pub fn summarize_humans(inputs: Vec<Human>) -> Result<HumanSummary> {
    let humans = inputs
        .into_iter()
        .map(normalize_human)
        .collect::<Result<Vec<_>>>()?;
    let duplicate_groups = find_duplicates(humans.clone())?.len();
    let mut identity_types = BTreeMap::new();
    let mut tags = BTreeMap::new();
    let mut identities = HashSet::new();
    let mut verified_identities = 0;
    let mut consent = ConsentSummary {
        email: 0,
        sms: 0,
        profiling: 0,
    };
    for human in &humans {
        for identity in &human.identities {
            *identity_types.entry(identity.kind.clone()).or_default() += 1;
            identities.insert(format!("{}:{}", identity.kind, identity.value));
            verified_identities += usize::from(identity.verified);
        }
        for tag in &human.tags {
            *tags.entry(tag.clone()).or_default() += 1;
        }
        consent.email += usize::from(human.consent.email);
        consent.sms += usize::from(human.consent.sms);
        consent.profiling += usize::from(human.consent.profiling);
    }
    Ok(HumanSummary {
        humans: humans.len(),
        unique_identities: identities.len(),
        verified_identities,
        duplicate_groups,
        identity_types: count_entries(identity_types),
        tags: count_entries(tags),
        consent,
    })
}
