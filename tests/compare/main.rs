//! `humans diff` through the built binary: one record added, one removed,
//! one whose name changed and one the same once normalised; and the refusal
//! for a snapshot holding one id twice.

use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::Command;

fn diff(dir: &PathBuf, before: Value, after: Value) -> std::process::Output {
    std::fs::create_dir_all(dir).expect("the test's directory");
    let (earlier, later) = (dir.join("before.json"), dir.join("after.json"));
    std::fs::write(&earlier, before.to_string()).expect("the earlier snapshot");
    std::fs::write(&later, after.to_string()).expect("the later snapshot");
    Command::new(env!("CARGO_BIN_EXE_humans-cli"))
        .args(["diff", "--before"])
        .arg(&earlier)
        .arg("--after")
        .arg(&later)
        .output()
        .expect("the binary runs")
}

#[test]
fn two_snapshots_say_what_was_added_removed_and_changed() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("diff-snapshots");
    let answer = diff(
        &dir,
        json!([
            {"id": "ada", "displayName": "Ada"},
            {"id": "bo", "displayName": "Bo", "tags": ["Customer"]},
            {"id": "cy", "displayName": "Cy"}
        ]),
        json!([
            {"id": "ada", "displayName": "Ada Lovelace"},
            {"id": "bo", "displayName": "Bo", "tags": ["customer"]},
            {"id": "di", "displayName": "Di"}
        ]),
    );
    assert!(answer.status.success(), "{}", String::from_utf8_lossy(&answer.stderr));
    let body: Value = serde_json::from_slice(&answer.stdout).expect("JSON on stdout");
    assert_eq!(body["added"], json!(["di"]));
    assert_eq!(body["removed"], json!(["cy"]));
    let one = std::slice::from_ref(&body).len();
    assert_eq!(body["unchanged"], json!(one), "Bo differs only in what normalisation lowercases");
    let changed = body["changed"].as_array().expect("the changed records");
    assert_eq!(changed.len(), one);
    assert_eq!(changed[one - one]["id"], "ada");
    assert_eq!(changed[one - one]["fields"]["displayName"], json!({"before": "Ada", "after": "Ada Lovelace"}));
}

#[test]
fn a_snapshot_holding_one_id_twice_is_refused() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("diff-twice");
    let answer = diff(&dir, json!([{"id": "ada"}, {"id": "ada"}]), json!([]));
    assert!(!answer.status.success(), "two records with one id are refused");
    let said = String::from_utf8_lossy(&answer.stderr);
    assert!(said.contains("before snapshot holds two records with id ada"), "{said}");
}
