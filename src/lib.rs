//! Human records: one shape for a person, and the five questions asked of a
//! set of them — normalise, deduplicate, merge, segment, summarise.
//!
//! The shapes and the operations live in `records`; this file is the surface
//! the CLI and any other caller uses, so the crate's API is one list.

pub mod records;

pub use records::{
    Consent, ConsentSummary, CountEntry, DuplicateGroup, Human, HumanSummary, Identity,
    MergeResult, Segment,
};
pub use records::dedupe::{find_duplicates, merge_humans};
pub use records::normalize::normalize_human;
pub use records::segments::{matches_segment, segment_humans};
pub use records::summary::summarize_humans;
