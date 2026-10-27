//! `humans tag` through the built binary: a tag put on every record a
//! segment selects, the others left alone, then taken off again; a record
//! that carries it already is not counted as changed, and asking for both
//! or neither of --add and --remove is refused.

use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::{Command, Output};

fn tag(humans: &PathBuf, segment: &PathBuf, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_humans-cli"))
        .args(["tag", "--humans"])
        .arg(humans)
        .arg("--segment")
        .arg(segment)
        .args(args)
        .output()
        .expect("the binary runs")
}

fn body(answer: Output) -> Value {
    assert!(answer.status.success(), "{}", String::from_utf8_lossy(&answer.stderr));
    serde_json::from_slice(&answer.stdout).expect("JSON on stdout")
}

#[test]
fn a_segment_is_tagged_and_untagged() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("tag-segment");
    std::fs::create_dir_all(&dir).expect("the test's directory");
    let humans = dir.join("humans.json");
    std::fs::write(
        &humans,
        json!([
            {"id": "ada", "tags": ["customer"]},
            {"id": "bo", "tags": ["customer", "vip"]},
            {"id": "cy", "tags": ["lead"]}
        ])
        .to_string(),
    )
    .expect("the records");
    let customers = dir.join("customers.json");
    std::fs::write(&customers, json!({"allTags": ["customer"]}).to_string()).expect("the segment");

    let added = body(tag(&humans, &customers, &["--add", "VIP"]));
    assert_eq!(added["tag"], "vip", "kept lowercased, as normalize keeps tags");
    assert_eq!(added["changedIds"], json!(["ada"]), "Bo carries it already; Cy is no customer");
    let one = std::slice::from_ref(&added).len();
    assert_eq!(added["humans"][one - one]["tags"], json!(["customer", "vip"]));
    assert_eq!(added["humans"][one + one]["tags"], json!(["lead"]));

    std::fs::write(&humans, added["humans"].to_string()).expect("the set written back");
    let removed = body(tag(&humans, &customers, &["--remove", "vip"]));
    assert_eq!(removed["added"], json!(false));
    assert_eq!(removed["changedIds"], json!(["ada", "bo"]));

    let neither = tag(&humans, &customers, &[]);
    assert!(!neither.status.success(), "a tag change names the tag");
    assert!(String::from_utf8_lossy(&neither.stderr).contains("tag needs exactly one of --add <tag> or --remove <tag>"));
}
