//! Segments: whether one record matches a declared segment, and the subset
//! of a set that does.

use std::collections::BTreeMap;

use anyhow::Result;

use super::{Consent, Human, Segment};
use super::normalize::normalize_human;

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
