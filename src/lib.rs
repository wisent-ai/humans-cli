//! Human records: one shape for a person, and the five questions asked of a
//! set of them — normalise, deduplicate, merge, segment, summarise.
//!
//! The shapes and the operations live in `records`; this file is the surface
//! the CLI and any other caller uses, so the crate's API is one list.

pub mod audience;
pub mod compare;
pub mod consolidate;
pub mod freshness;
pub mod records;
pub mod sheet_read;
pub mod vcard;
pub mod tagging;

pub use records::{
    Consent, ConsentSummary, CountEntry, DuplicateGroup, Human, HumanSummary, Identity,
    MergeResult, Segment,
};
pub use compare::{Change, Changed, Difference, diff_humans};
pub use audience::{Audience, Channel, Recipient, audience};
pub use consolidate::{Consolidated, Consolidation, consolidate_humans};
pub use freshness::{Freshness, StaleRecord, stale_humans};
pub use vcard::{Cards, humans_vcards};
pub use tagging::{Tagged, tag_segment};
pub use records::dedupe::{find_duplicates, merge_humans};
pub use records::normalize::normalize_human;
pub use records::sheet::humans_sheet;
pub use sheet_read::humans_from_sheet;
pub use records::segments::{matches_segment, segment_humans};
pub use records::summary::summarize_humans;
