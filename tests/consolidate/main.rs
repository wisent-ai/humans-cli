//! `humans consolidate` through the built binary: three records joined in a
//! chain (an email shared by the first two, a phone by the last two) become
//! one person built on the first, a fourth stays as it was, and the answer
//! says which records were merged.

use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::Command;

#[test]
fn records_joined_through_any_shared_identity_become_one_person() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("consolidate-chain");
    std::fs::create_dir_all(&dir).expect("the test's directory");
    let file = dir.join("humans.json");
    let humans = json!([
        {"id": "ada", "displayName": "Ada", "identities": [{"type": "email", "value": "ada@falcon.example"}]},
        {"id": "ada-crm", "identities": [{"type": "email", "value": "ada@falcon.example"}, {"type": "phone", "value": "+48500100200"}], "tags": ["customer"]},
        {"id": "ada-phone", "identities": [{"type": "phone", "value": "+48500100200"}]},
        {"id": "bo", "displayName": "Bo", "identities": [{"type": "email", "value": "bo@falcon.example"}]}
    ]);
    std::fs::write(&file, humans.to_string()).expect("the input file");

    let answer = Command::new(env!("CARGO_BIN_EXE_humans-cli"))
        .args(["consolidate", "--humans"])
        .arg(&file)
        .output()
        .expect("the binary runs");
    assert!(answer.status.success(), "{}", String::from_utf8_lossy(&answer.stderr));
    let body: Value = serde_json::from_slice(&answer.stdout).expect("JSON on stdout");

    let ids: Vec<&str> = body["humans"].as_array().expect("the set").iter().map(|human| human["id"].as_str().expect("an id")).collect();
    assert_eq!(ids, vec!["ada", "bo"], "the chain became Ada, built on the first record; Bo stayed");
    let merged = body["merged"].as_array().expect("what was merged");
    let one = std::slice::from_ref(&body).len();
    assert_eq!(merged.len(), one, "one person was merged");
    assert_eq!(merged[one - one]["id"], "ada");
    assert_eq!(merged[one - one]["sourceIds"], json!(["ada", "ada-crm", "ada-phone"]));
    let ada = &body["humans"][one - one];
    assert_eq!(ada["tags"], json!(["customer"]), "the merged person keeps the tags of every record");
    assert_eq!(ada["identities"].as_array().expect("identities").len(), one + one, "the email and the phone");
}
