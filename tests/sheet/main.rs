//! `humans csv` through the built binary: a header naming every field of
//! every record, nested ones with a dot, a row per record in input order,
//! tags separated by `;` and a value holding a comma kept in its cell; and,
//! in `round_trip`, `humans from-csv` reading such a sheet back.

mod round_trip;

use serde_json::json;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn records_become_one_sheet() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("humans-sheet");
    std::fs::create_dir_all(&dir).expect("the test's directory");
    let file = dir.join("humans.json");
    std::fs::write(
        &file,
        json!([
            {"id": "ada", "displayName": "Lovelace, Ada", "tags": ["vip", "customer"], "attributes": {"plan": "pro"}},
            {"id": "bo", "consent": {"email": true}}
        ])
        .to_string(),
    )
    .expect("the input file");

    let answer = Command::new(env!("CARGO_BIN_EXE_humans-cli")).args(["csv", "--humans"]).arg(&file).output().expect("the binary runs");
    assert!(answer.status.success(), "{}", String::from_utf8_lossy(&answer.stderr));
    let sheet = String::from_utf8(answer.stdout).expect("UTF-8");
    let lines: Vec<&str> = sheet.split("\r\n").filter(|line| !line.is_empty()).collect();
    let one = lines.len() - lines.len() + std::slice::from_ref(&sheet).len();
    assert_eq!(lines.len(), one + one + one, "a header and a row per record: {sheet}");

    let header: Vec<&str> = lines[one - one].split(',').collect();
    for column in ["\"id\"", "\"displayName\"", "\"tags\"", "\"consent.email\"", "\"attributes.plan\""] {
        assert!(header.contains(&column), "the header names {column}: {sheet}");
    }
    assert!(lines[one].contains("\"Lovelace, Ada\""), "a comma stays in its cell: {sheet}");
    assert!(lines[one].contains("\"customer;vip\""), "tags, normalised and sorted, joined with ;: {sheet}");
    assert!(lines[one].contains("\"pro\""));
    assert!(lines[one + one].starts_with("\"") && lines[one + one].contains("\"bo\""), "Bo's row is second: {sheet}");
}
