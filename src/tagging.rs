//! Tagging a segment: putting a label on, or taking it off, every record a
//! segment selects, and answering with the whole set so it can be written
//! back. A record the segment does not select is left as it was, normalised.

use anyhow::{Result, bail};
use serde::Serialize;

use crate::records::normalize::{normalize_human, normalize_tags};
use crate::records::segments::matches_segment;
use crate::records::{Human, Segment};

/// What a tag change did.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tagged {
    pub tag: String,
    /// Whether the tag was put on (`true`) or taken off (`false`).
    pub added: bool,
    /// The ids of the records whose tags changed, in input order.
    pub changed_ids: Vec<String>,
    /// Every record, the changed ones with their new tags.
    pub humans: Vec<Human>,
}

/// Puts `tag` on every record `segment` selects when `add`, or takes it off.
pub fn tag_segment(inputs: Vec<Human>, segment: &Segment, tag: &str, add: bool) -> Result<Tagged> {
    let Some(tag) = normalize_tags(vec![tag.to_owned()])?.into_iter().next() else {
        bail!("the tag is empty; name the label to put on or take off");
    };
    let mut changed_ids = Vec::new();
    let mut humans = Vec::new();
    for input in inputs {
        let mut human = normalize_human(input)?;
        if matches_segment(&human, segment) && human.tags.contains(&tag) != add {
            if add {
                human.tags.push(tag.clone());
                human.tags.sort();
            } else {
                human.tags.retain(|held| held != &tag);
            }
            changed_ids.push(human.id.clone());
        }
        humans.push(human);
    }
    Ok(Tagged { tag, added: add, changed_ids, humans })
}
