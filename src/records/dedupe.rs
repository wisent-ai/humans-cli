//! Duplicates: which records describe the same person, and what merging
//! them produces — including the conflicts a merge refuses to invent an
//! answer for.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use anyhow::{Result, bail};

use super::{DuplicateGroup, Human, Identity, MergeResult};
use super::normalize::{normalize_human, required};

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
