//! A set of records as one CSV sheet for a spreadsheet: a row per record,
//! normalised. The columns are the record's own fields as its JSON names
//! them, nested ones joined with a dot (`consent.email`,
//! `attributes.plan`), the union over every record, by name. A list of
//! plain values is written separated by `;`; a list of objects (the
//! identities) as compact JSON, so nothing about it is lost. Every cell is
//! quoted (RFC 4180), so a comma, a quote or a line break stays in its cell.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use serde_json::Value;

use super::normalize::normalize_human;
use super::Human;

fn cell(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn line<'a>(cells: impl Iterator<Item = &'a str>) -> String {
    format!("{}\r\n", cells.map(cell).collect::<Vec<_>>().join(","))
}

/// A plain value as its cell reads it; `null` is an empty cell.
fn plain(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

/// `value`'s cells under `prefix`, nested objects flattened.
fn flatten(prefix: &str, value: &Value, cells: &mut BTreeMap<String, String>) {
    match value {
        Value::Object(fields) => {
            for (name, field) in fields {
                let path = if prefix.is_empty() { name.clone() } else { format!("{prefix}.{name}") };
                flatten(&path, field, cells);
            }
        }
        Value::Array(items) if items.iter().all(|item| !item.is_object() && !item.is_array()) => {
            cells.insert(prefix.to_owned(), items.iter().map(plain).collect::<Vec<_>>().join(";"));
        }
        Value::Array(_) => {
            cells.insert(prefix.to_owned(), value.to_string());
        }
        other => {
            cells.insert(prefix.to_owned(), plain(other));
        }
    }
}

/// Every record of `inputs` as a row, in input order, under the header.
pub fn humans_sheet(inputs: Vec<Human>) -> Result<String> {
    let mut rows = Vec::new();
    let mut columns = BTreeSet::new();
    for input in inputs {
        let human = normalize_human(input)?;
        let mut cells = BTreeMap::new();
        flatten("", &serde_json::to_value(&human)?, &mut cells);
        columns.extend(cells.keys().cloned());
        rows.push(cells);
    }
    let mut sheet = line(columns.iter().map(String::as_str));
    for cells in &rows {
        sheet.push_str(&line(columns.iter().map(|column| match cells.get(column) {
            Some(text) => text.as_str(),
            None => "",
        })));
    }
    Ok(sheet)
}
