//! Comparing two snapshots of a set of records, an export from last week
//! and one from today: which ids were added, which were removed, and for
//! each id in both, the fields whose normalised values differ, nested
//! fields named with a dot. Two records that differ only in what
//! normalisation removes (case, spaces, order) are the same.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Result, bail};
use serde::Serialize;
use serde_json::Value;

use crate::records::Human;
use crate::records::normalize::normalize_human;

/// One field's value before and after; `null` on the side that lacks it.
#[derive(Debug, Clone, Serialize)]
pub struct Change {
    pub before: Option<Value>,
    pub after: Option<Value>,
}

/// One record whose fields changed.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Changed {
    pub id: String,
    /// Every field that differs, by its dotted name.
    pub fields: BTreeMap<String, Change>,
}

/// What changed between two snapshots.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Difference {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<Changed>,
    pub unchanged: usize,
}

/// Every record of `inputs` normalised, by id; two with one id are refused.
fn by_id(inputs: Vec<Human>, side: &str) -> Result<BTreeMap<String, Value>> {
    let mut found = BTreeMap::new();
    for input in inputs {
        let human = normalize_human(input)?;
        let id = human.id.clone();
        if found.insert(id.clone(), serde_json::to_value(&human)?).is_some() {
            bail!("the {side} snapshot holds two records with id {id}; an id names one record");
        }
    }
    Ok(found)
}

/// Every leaf of `value` under `prefix`, objects flattened with a dot.
fn leaves(prefix: &str, value: &Value, into: &mut BTreeMap<String, Value>) {
    match value {
        Value::Object(fields) => {
            for (name, field) in fields {
                let path = if prefix.is_empty() { name.clone() } else { format!("{prefix}.{name}") };
                leaves(&path, field, into);
            }
        }
        other => {
            into.insert(prefix.to_owned(), other.clone());
        }
    }
}

/// What changed from `before` to `after`.
pub fn diff_humans(before: Vec<Human>, after: Vec<Human>) -> Result<Difference> {
    let before = by_id(before, "before")?;
    let after = by_id(after, "after")?;
    let added = after.keys().filter(|id| !before.contains_key(*id)).cloned().collect();
    let removed = before.keys().filter(|id| !after.contains_key(*id)).cloned().collect();
    let mut changed = Vec::new();
    let mut unchanged = Vec::new();
    for (id, old) in &before {
        let Some(new) = after.get(id) else {
            continue;
        };
        let (mut was, mut now) = (BTreeMap::new(), BTreeMap::new());
        leaves("", old, &mut was);
        leaves("", new, &mut now);
        let names: BTreeSet<&String> = was.keys().chain(now.keys()).collect();
        let fields: BTreeMap<String, Change> = names
            .into_iter()
            .filter_map(|name| {
                let change = Change { before: was.get(name).cloned(), after: now.get(name).cloned() };
                (change.before != change.after).then(|| (name.clone(), change))
            })
            .collect();
        if fields.is_empty() {
            unchanged.push(id.clone());
        } else {
            changed.push(Changed { id: id.clone(), fields });
        }
    }
    Ok(Difference { added, removed, changed, unchanged: unchanged.len() })
}
