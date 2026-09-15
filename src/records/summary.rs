//! Summarising a set of records: how many of each value, and what the
//! consent state adds up to.

use std::collections::{BTreeMap, HashSet};

use anyhow::Result;

use super::{ConsentSummary, CountEntry, Human, HumanSummary};
use super::dedupe::find_duplicates;
use super::normalize::normalize_human;

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
