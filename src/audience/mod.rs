//! Who may be reached on a channel: the people of a set who consented to it
//! and have an address for it, each address once, and how many were left out
//! and why, so a send list never holds someone who did not agree to it.
//!
//! Email needs `consent.email` and an `email` identity; SMS needs
//! `consent.sms` and a `phone` identity. A verified address is taken before an
//! unverified one; `--verified-only` takes only verified addresses.

use std::collections::BTreeSet;

use anyhow::Result;
use serde::Serialize;

use crate::records::Human;
use crate::records::normalize::normalize_human;

/// A channel people are reached on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Email,
    Sms,
}

impl Channel {
    /// The identity type an address on this channel is.
    fn address_kind(self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Sms => "phone",
        }
    }

    fn consented(self, human: &Human) -> bool {
        match self {
            Self::Email => human.consent.email,
            Self::Sms => human.consent.sms,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Recipient {
    pub id: String,
    pub display_name: Option<String>,
    pub address: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Audience {
    pub channel: Channel,
    /// One per address, in input order; a second person with an address
    /// already listed is counted under `sharedAddress`.
    pub recipients: Vec<Recipient>,
    pub without_consent: Vec<String>,
    pub without_address: Vec<String>,
    pub shared_address: Vec<String>,
}

/// The people of `humans` reachable on `channel`.
pub fn audience(humans: Vec<Human>, channel: Channel, verified_only: bool) -> Result<Audience> {
    let mut seen = BTreeSet::new();
    let mut result = Audience {
        channel,
        recipients: Vec::new(),
        without_consent: Vec::new(),
        without_address: Vec::new(),
        shared_address: Vec::new(),
    };
    for input in humans {
        let human = normalize_human(input)?;
        if !channel.consented(&human) {
            result.without_consent.push(human.id);
            continue;
        }
        let mut addresses: Vec<_> = human
            .identities
            .iter()
            .filter(|identity| identity.kind == channel.address_kind())
            .filter(|identity| identity.verified || !verified_only)
            .collect();
        // Verified first, so the address taken is the one known to work.
        addresses.sort_by_key(|identity| !identity.verified);
        let Some(identity) = addresses.first() else {
            result.without_address.push(human.id);
            continue;
        };
        if !seen.insert(identity.value.clone()) {
            result.shared_address.push(human.id);
            continue;
        }
        result.recipients.push(Recipient {
            id: human.id.clone(),
            display_name: human.display_name.clone(),
            address: identity.value.clone(),
            verified: identity.verified,
        });
    }
    Ok(result)
}
