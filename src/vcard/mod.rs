//! The records as vCards (RFC 6350), one card per person, for an address
//! book or a phone: the name, every email and phone identity (a verified one
//! marked preferred), and the tags as categories. A record with neither a
//! name nor an address has nothing an address book can show and is named
//! under `skipped` rather than written as an empty card.

use anyhow::Result;
use serde::Serialize;

use crate::records::Human;
use crate::records::normalize::normalize_human;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cards {
    /// The cards, in input order, joined as one `.vcf` file.
    pub vcf: String,
    pub written: usize,
    pub skipped: Vec<String>,
}

/// `text` with the characters vCard reserves escaped (RFC 6350 section 3.4).
fn escape(text: &str) -> String {
    text.replace('\\', "\\\\").replace(',', "\\,").replace(';', "\\;").replace('\n', "\\n")
}

/// One card's content lines, or none when the record has nothing to show.
fn card(human: &Human) -> Option<Vec<String>> {
    let given = human.given_name.as_deref().map(str::trim).filter(|name| !name.is_empty());
    let family = human.family_name.as_deref().map(str::trim).filter(|name| !name.is_empty());
    let display = match (&human.display_name, given, family) {
        (Some(name), _, _) if !name.trim().is_empty() => Some(name.trim().to_owned()),
        (_, Some(given), Some(family)) => Some(format!("{given} {family}")),
        (_, Some(only), None) | (_, None, Some(only)) => Some(only.to_owned()),
        _ => None,
    };
    let addresses: Vec<String> = human
        .identities
        .iter()
        .filter_map(|identity| {
            let preferred = if identity.verified { ";PREF=1" } else { "" };
            match identity.kind.as_str() {
                "email" => Some(format!("EMAIL{preferred}:{}", escape(&identity.value))),
                "phone" => Some(format!("TEL;VALUE=uri{preferred}:tel:{}", identity.value.replace(' ', ""))),
                _ => None,
            }
        })
        .collect();
    let formatted = match (&display, addresses.first()) {
        (Some(name), _) => name.clone(),
        (None, Some(_)) => human.id.clone(),
        (None, None) => return None,
    };
    let mut lines = vec!["BEGIN:VCARD".to_owned(), "VERSION:4.0".to_owned(), format!("UID:{}", escape(&human.id))];
    // The structured name: family; given; (additional, prefix, suffix unstated).
    let part = |name: Option<&str>| match name {
        Some(name) => escape(name),
        None => String::new(),
    };
    lines.push(format!("N:{};{};;;", part(family), part(given)));
    lines.extend(addresses);
    if !human.tags.is_empty() {
        lines.push(format!("CATEGORIES:{}", human.tags.iter().map(|tag| escape(tag)).collect::<Vec<_>>().join(",")));
    }
    lines.push("END:VCARD".to_owned());
    Some(lines)
}

/// Every record of `humans`, normalised, as a vCard.
pub fn humans_vcards(humans: Vec<Human>) -> Result<Cards> {
    let mut vcf = String::new();
    let mut written = Vec::new();
    let mut skipped = Vec::new();
    for input in humans {
        let human = normalize_human(input)?;
        match card(&human) {
            Some(lines) => {
                // vCard lines end in CRLF (RFC 6350 section 3.2).
                vcf.push_str(&lines.join("\r\n"));
                vcf.push_str("\r\n");
                written.push(human.id);
            }
            None => skipped.push(human.id),
        }
    }
    Ok(Cards { vcf, written: written.len(), skipped })
}
