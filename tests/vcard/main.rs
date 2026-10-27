//! `humans vcard` through the built binary: a person with a name, a
//! verified email, an unverified phone and a tag holding a comma; one with
//! only an email; and one with nothing to put on a card, which is skipped.

use serde_json::json;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn people_become_cards_and_an_empty_record_is_skipped() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("vcard");
    std::fs::create_dir_all(&dir).expect("the test's directory");
    let file = dir.join("humans.json");
    std::fs::write(
        &file,
        json!([
            {"id": "ada", "givenName": "Ada", "familyName": "Lovelace", "tags": ["engines, analytical"],
             "identities": [{"type": "email", "value": "ada@example.org", "verified": true}, {"type": "phone", "value": "+44 20 7946 0000"}]},
            {"id": "bo", "identities": [{"type": "email", "value": "bo@example.org"}]},
            {"id": "nobody"}
        ])
        .to_string(),
    )
    .expect("the people");

    let answer = Command::new(env!("CARGO_BIN_EXE_humans-cli")).args(["vcard", "--humans"]).arg(&file).output().expect("the binary runs");
    assert!(answer.status.success(), "{}", String::from_utf8_lossy(&answer.stderr));
    let vcf = String::from_utf8(answer.stdout).expect("UTF-8");
    assert_eq!(vcf.matches("BEGIN:VCARD\r\n").count(), vcf.matches("END:VCARD\r\n").count());
    assert!(vcf.contains("UID:ada\r\n") && vcf.contains("UID:bo\r\n"), "{vcf}");
    assert!(!vcf.contains("UID:nobody"), "a record with nothing to show gets no card: {vcf}");
    assert!(vcf.contains("FN:Ada Lovelace\r\n"), "{vcf}");
    assert!(vcf.contains("N:Lovelace;Ada;;;\r\n"), "{vcf}");
    assert!(vcf.contains("EMAIL;PREF=1:ada@example.org\r\n"), "the verified email is preferred: {vcf}");
    assert!(vcf.contains("TEL;VALUE=uri:tel:+442079460000\r\n"), "{vcf}");
    assert!(vcf.contains("CATEGORIES:engines\\, analytical\r\n"), "a comma in a tag is escaped: {vcf}");
    assert!(vcf.contains("FN:bo\r\n"), "a record with only an address is named by its id: {vcf}");
    assert!(String::from_utf8_lossy(&answer.stderr).contains("skipped nobody"));
}
