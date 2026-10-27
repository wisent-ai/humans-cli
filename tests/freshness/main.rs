//! `humans stale` through the built binary: a record updated long ago, one
//! never updated but created long ago, one updated recently, and one stating
//! no date; then a cutoff that is not RFC 3339.

use chrono::{TimeDelta, Utc};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::{Command, Output};

fn stale(file: &PathBuf, before: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_humans-cli"))
        .args(["stale", "--humans"])
        .arg(file)
        .args(["--before", before])
        .output()
        .expect("the binary runs")
}

#[test]
fn records_untouched_since_the_cutoff_are_listed_longest_first() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("stale");
    std::fs::create_dir_all(&dir).expect("the test's directory");
    let now = Utc::now();
    let one = std::slice::from_ref(&dir).len() as i32;
    let week = TimeDelta::weeks(one.into());
    // Weeks ago: the cutoff one, "old" a week's days (seven), "created-only" seven times that.
    let days = week.num_days() as i32;
    let ago = |weeks: i32| (now - week * weeks).to_rfc3339();
    let cutoff = ago(one);
    let file = dir.join("humans.json");
    std::fs::write(
        &file,
        json!([
            {"id": "recent", "updatedAt": now.to_rfc3339()},
            {"id": "old", "updatedAt": ago(days)},
            {"id": "created-only", "createdAt": ago(days * days)},
            {"id": "undated"}
        ])
        .to_string(),
    )
    .expect("the people");

    let answer = stale(&file, &cutoff);
    assert!(answer.status.success(), "{}", String::from_utf8_lossy(&answer.stderr));
    let result: Value = serde_json::from_slice(&answer.stdout).expect("JSON on stdout");
    let ids: Vec<&str> = result["stale"].as_array().expect("stale").iter().map(|item| item["id"].as_str().expect("an id")).collect();
    assert_eq!(ids, vec!["created-only", "old"], "the longest untouched first, the recent one left out");
    assert_eq!(result["undated"], json!(["undated"]), "a record with no date is listed apart");
    assert_eq!(result["fresh"], std::slice::from_ref(&dir).len());

    let wrong = stale(&file, "last tuesday");
    assert!(!wrong.status.success(), "a cutoff that is not RFC 3339 is refused");
}
