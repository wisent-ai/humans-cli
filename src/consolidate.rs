//! Consolidating a whole set: every group of records that describe one
//! person, joined through any identity they share (an email shared by A and
//! B and a phone shared by B and C make A, B and C one person), merged the
//! way `merge` merges, the record seen first in the input as the base. The
//! answer is the deduplicated set in input order and, per merged person,
//! which records went into it and the fields that conflicted.

use std::collections::HashMap;

use anyhow::Result;
use serde::Serialize;

use crate::records::dedupe::merge_humans;
use crate::records::normalize::normalize_human;
use crate::records::Human;

/// One person several records were merged into.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Consolidated {
    pub id: String,
    pub source_ids: Vec<String>,
    pub conflicts: Vec<String>,
}

/// The deduplicated set and what was merged to make it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Consolidation {
    pub humans: Vec<Human>,
    pub merged: Vec<Consolidated>,
}

/// The representative of `index`'s group, the earliest record in it.
fn root(parents: &mut [usize], index: usize) -> usize {
    let mut current = index;
    while parents[current] != current {
        parents[current] = parents[parents[current]];
        current = parents[current];
    }
    current
}

fn join(parents: &mut [usize], left: usize, right: usize) {
    let (left, right) = (root(parents, left), root(parents, right));
    // The earlier record stays the representative, so it stays the base.
    if left < right {
        parents[right] = left;
    } else {
        parents[left] = right;
    }
}

/// Every record of `inputs` normalised, those sharing an identity merged.
pub fn consolidate_humans(inputs: Vec<Human>) -> Result<Consolidation> {
    let humans = inputs.into_iter().map(normalize_human).collect::<Result<Vec<_>>>()?;
    // Each record starts as its own group, its own representative.
    let mut parents: Vec<usize> = humans.iter().enumerate().map(|(index, _)| index).collect();
    let mut seen: HashMap<String, usize> = HashMap::new();
    for (index, human) in humans.iter().enumerate() {
        for identity in &human.identities {
            let key = format!("{}:{}", identity.kind, identity.value);
            match seen.get(&key) {
                Some(&first) => join(&mut parents, first, index),
                None => {
                    seen.insert(key, index);
                }
            }
        }
    }
    let mut groups: Vec<(usize, Vec<Human>)> = Vec::new();
    for (index, human) in humans.into_iter().enumerate() {
        let representative = root(&mut parents, index);
        match groups.iter_mut().find(|(held, _)| *held == representative) {
            Some((_, members)) => members.push(human),
            None => groups.push((representative, vec![human])),
        }
    }
    let mut kept = Vec::new();
    let mut merged = Vec::new();
    for (_, members) in groups {
        if members.len() == std::slice::from_ref(&members).len() {
            kept.extend(members);
            continue;
        }
        let result = merge_humans(members, None)?;
        merged.push(Consolidated {
            id: result.human.id.clone(),
            source_ids: result.source_ids,
            conflicts: result.conflicts,
        });
        kept.push(result.human);
    }
    Ok(Consolidation { humans: kept, merged })
}
