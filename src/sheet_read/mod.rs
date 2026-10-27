//! A sheet back into records: the inverse of `humans csv`, so a set exported
//! to a spreadsheet, edited there and saved as CSV comes back as records.
//! The header names fields as `csv` writes them, nested ones joined with a
//! dot. Which columns are lists, and which are nested objects, is read from
//! the record's own shape (a record holding only an id, serialised), not from
//! a list kept here: a list cell is compact JSON when it starts with `[`
//! (the identities), else values separated by `;` (the tags). A nested cell
//! that reads as a JSON boolean or number is that value, else text; an empty
//! cell is a field the row does not set. Every row is normalised as
//! `normalize` would, and a malformed sheet is refused naming the row.

use anyhow::{Context, Result, bail};
use serde_json::{Map, Value, json};

use crate::records::Human;
use crate::records::normalize::normalize_human;

/// The cells of `text`, row by row, as RFC 4180 reads them.
fn rows(text: &str) -> Result<Vec<Vec<String>>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut cell = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    while let Some(character) = chars.next() {
        match (quoted, character) {
            (true, '"') if chars.peek() == Some(&'"') => {
                chars.next();
                cell.push('"');
            }
            (true, '"') => quoted = false,
            (true, other) => cell.push(other),
            (false, '"') if cell.is_empty() => quoted = true,
            (false, ',') => row.push(std::mem::take(&mut cell)),
            (false, '\r') if chars.peek() == Some(&'\n') => {}
            (false, '\n') => {
                row.push(std::mem::take(&mut cell));
                rows.push(std::mem::take(&mut row));
            }
            (false, other) => cell.push(other),
        }
    }
    if quoted {
        bail!("the sheet ends inside a quoted cell; a closing quote is missing");
    }
    if !cell.is_empty() || !row.is_empty() {
        row.push(cell);
        rows.push(row);
    }
    Ok(rows)
}

/// A nested cell: a JSON boolean or number when it reads as one, else text.
fn scalar(cell: &str) -> Value {
    match serde_json::from_str::<Value>(cell) {
        Ok(value @ (Value::Bool(_) | Value::Number(_))) => value,
        _ => Value::String(cell.to_owned()),
    }
}

/// Put `value` at the dotted `path` of `record`.
fn place(record: &mut Map<String, Value>, path: &str, value: Value) -> Result<()> {
    let mut parts: Vec<&str> = path.split('.').collect();
    let Some(last) = parts.pop() else {
        bail!("an empty column name");
    };
    let mut current = record;
    for part in parts {
        let entry = current.entry(part.to_owned()).or_insert_with(|| json!({}));
        let Value::Object(next) = entry else {
            bail!("column {path} nests under {part}, which another column already set to a value");
        };
        current = next;
    }
    current.insert(last.to_owned(), value);
    Ok(())
}

/// The records a `humans csv` sheet holds, normalised, in row order.
pub fn humans_from_sheet(text: &str) -> Result<Vec<Human>> {
    let shape = serde_json::to_value(serde_json::from_value::<Human>(json!({"id": "shape"}))?)?;
    let mut all = rows(text)?.into_iter();
    let Some(header) = all.next() else {
        bail!("the sheet is empty; its first row must name the columns, as humans csv writes them");
    };
    let mut humans = Vec::new();
    for (index, cells) in all.enumerate() {
        // Lines count from one, and the header is the first.
        let one = std::slice::from_ref(&header).len();
        let line = index + one + one;
        if cells.len() != header.len() {
            bail!("row {line} has {} cells under a header of {}", cells.len(), header.len());
        }
        let mut record = Map::new();
        for (column, cell) in header.iter().zip(&cells).filter(|(_, cell)| !cell.is_empty()) {
            let top = column.split('.').next().context("an empty column name")?;
            let value = match (shape.get(top), column.contains('.')) {
                (Some(Value::Array(_)), _) if cell.starts_with('[') => {
                    serde_json::from_str(cell).with_context(|| format!("row {line}, column {column}: a list that is not JSON"))?
                }
                (Some(Value::Array(_)), _) => Value::Array(cell.split(';').map(|item| Value::String(item.to_owned())).collect()),
                (_, true) => scalar(cell),
                _ => Value::String(cell.clone()),
            };
            place(&mut record, column, value).with_context(|| format!("row {line}"))?;
        }
        let human: Human = serde_json::from_value(Value::Object(record)).with_context(|| format!("row {line} is not a record"))?;
        humans.push(normalize_human(human).with_context(|| format!("row {line}"))?);
    }
    Ok(humans)
}
