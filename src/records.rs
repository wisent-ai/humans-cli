//! What a human record is, and what an operation over a set of them
//! answers with.
//!
//! The operations themselves are the submodules: normalising one record,
//! finding and merging duplicates, matching a segment, and summarising a
//! set.

pub mod dedupe;
pub mod normalize;
pub mod segments;
pub mod summary;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    #[serde(rename = "type")]
    pub kind: String,
    pub value: String,
    #[serde(default)]
    pub verified: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Consent {
    #[serde(default)]
    pub email: bool,
    #[serde(default)]
    pub sms: bool,
    #[serde(default)]
    pub profiling: bool,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Human {
    pub id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub given_name: Option<String>,
    #[serde(default)]
    pub family_name: Option<String>,
    #[serde(default)]
    pub identities: Vec<Identity>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub attributes: BTreeMap<String, Value>,
    #[serde(default)]
    pub consent: Consent,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroup {
    pub key: String,
    pub identity_type: String,
    pub identity_value: String,
    pub human_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    #[serde(default)]
    pub all_tags: Vec<String>,
    #[serde(default)]
    pub any_tags: Vec<String>,
    #[serde(default)]
    pub identity_types: Vec<String>,
    #[serde(default)]
    pub require_verified_identity: bool,
    #[serde(default)]
    pub consent: BTreeMap<String, bool>,
    #[serde(default)]
    pub attributes: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeResult {
    pub human: Human,
    pub source_ids: Vec<String>,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanSummary {
    pub humans: usize,
    pub unique_identities: usize,
    pub verified_identities: usize,
    pub duplicate_groups: usize,
    pub identity_types: Vec<CountEntry>,
    pub tags: Vec<CountEntry>,
    pub consent: ConsentSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct CountEntry {
    pub key: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConsentSummary {
    pub email: usize,
    pub sms: usize,
    pub profiling: usize,
}
