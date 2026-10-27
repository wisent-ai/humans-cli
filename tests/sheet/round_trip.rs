//! `humans csv` then `humans from-csv` through the built binary: a set comes
//! back as the same records, identities, tags and consent included, and a
//! sheet whose row has a cell too many is refused naming its line.

use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::Command;

const BINARY: &str = env!("CARGO_BIN_EXE_humans-cli");

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::create_dir_all(&dir).expect("the test's directory");
    dir
}

#[test]
fn a_sheet_comes_back_as_the_records_it_was_written_from() {
    let dir = scratch("sheet-round-trip");
    let people = dir.join("people.json");
    std::fs::write(
        &people,
        json!([
            {"id": "ada", "displayName": "Ada, \"the first\"", "tags": ["customer", "vip"],
             "identities": [{"type": "email", "value": "ada@falcon.example"}]},
            {"id": "bo", "displayName": "Bo"}
        ])
        .to_string(),
    )
    .expect("the records");

    let sheet = Command::new(BINARY).args(["csv", "--humans"]).arg(&people).output().expect("csv runs");
    assert!(sheet.status.success(), "{}", String::from_utf8_lossy(&sheet.stderr));
    let csv = dir.join("people.csv");
    std::fs::write(&csv, &sheet.stdout).expect("the sheet");

    let normalized = Command::new(BINARY).args(["consolidate", "--humans"]).arg(&people).output().expect("consolidate runs");
    assert!(normalized.status.success());
    let expected: Value = serde_json::from_slice(&normalized.stdout).expect("JSON");

    let back = Command::new(BINARY).args(["from-csv", "--csv"]).arg(&csv).output().expect("from-csv runs");
    assert!(back.status.success(), "{}", String::from_utf8_lossy(&back.stderr));
    let records: Value = serde_json::from_slice(&back.stdout).expect("JSON on stdout");
    assert_eq!(records, expected["humans"], "the sheet held every field, quotes and commas included");
}

#[test]
fn a_row_with_a_cell_too_many_is_refused_naming_its_line() {
    let dir = scratch("sheet-ragged");
    let csv = dir.join("ragged.csv");
    std::fs::write(&csv, "\"id\",\"displayName\"\r\n\"ada\",\"Ada\",\"extra\"\r\n").expect("the sheet");
    let back = Command::new(BINARY).args(["from-csv", "--csv"]).arg(&csv).output().expect("from-csv runs");
    assert!(!back.status.success());
    let said = String::from_utf8_lossy(&back.stderr);
    assert!(said.contains("row 2 has 3 cells under a header of 2"), "{said}");
}
