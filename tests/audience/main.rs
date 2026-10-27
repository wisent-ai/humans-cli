//! `humans audience` through the built binary: of five people, one consented
//! with a verified and an unverified email (the verified one is taken), one
//! consented with an email someone earlier holds, one consented without an
//! email, one who did not consent, and one consented to SMS only; then the
//! same set with `--verified-only`, and a channel that does not exist.

use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::{Command, Output};

fn audience(file: &PathBuf, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_humans-cli"))
        .args(["audience", "--humans"])
        .arg(file)
        .args(extra)
        .output()
        .expect("the binary runs")
}

fn ids(list: &Value) -> Vec<&str> {
    list.as_array().expect("a list").iter().map(|item| item.as_str().expect("an id")).collect()
}

#[test]
fn consent_and_an_address_decide_who_is_reached() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("audience");
    std::fs::create_dir_all(&dir).expect("the test's directory");
    let email = |value: &str, verified: bool| json!({"type": "email", "value": value, "verified": verified});
    let file = dir.join("humans.json");
    std::fs::write(
        &file,
        json!([
            {"id": "ada", "identities": [email("ada@old.example", false), email("Ada@Example.org", true)], "consent": {"email": true}},
            {"id": "twin", "identities": [email("ada@example.org", false)], "consent": {"email": true}},
            {"id": "bo", "identities": [], "consent": {"email": true}},
            {"id": "cy", "identities": [email("cy@example.org", true)], "consent": {"email": false}},
            {"id": "di", "identities": [{"type": "phone", "value": "+48 600 100 200"}], "consent": {"sms": true}}
        ])
        .to_string(),
    )
    .expect("the people");

    let answer = audience(&file, &["--channel", "email"]);
    assert!(answer.status.success(), "{}", String::from_utf8_lossy(&answer.stderr));
    let result: Value = serde_json::from_slice(&answer.stdout).expect("JSON on stdout");
    let [ada] = result["recipients"].as_array().expect("recipients").as_slice() else { panic!("one recipient: {result}") };
    assert_eq!(ada["address"], "ada@example.org", "the verified address, normalised");
    assert_eq!(ada["verified"], true);
    assert_eq!(ids(&result["sharedAddress"]), vec!["twin"]);
    assert_eq!(ids(&result["withoutAddress"]), vec!["bo"]);
    assert_eq!(ids(&result["withoutConsent"]), vec!["cy", "di"]);

    let sms: Value = serde_json::from_slice(&audience(&file, &["--channel", "sms"]).stdout).expect("JSON on stdout");
    let reached: Vec<&str> = sms["recipients"].as_array().expect("recipients").iter().map(|item| item["id"].as_str().expect("an id")).collect();
    assert_eq!(reached, vec!["di"], "only the person who consented to SMS and has a phone");

    let verified: Value = serde_json::from_slice(&audience(&file, &["--channel", "email", "--verified-only"]).stdout).expect("JSON");
    assert_eq!(ids(&verified["withoutAddress"]), vec!["twin", "bo"], "an unverified address does not count");

    let wrong = audience(&file, &["--channel", "post"]);
    assert!(!wrong.status.success(), "a channel that does not exist is refused");
}
