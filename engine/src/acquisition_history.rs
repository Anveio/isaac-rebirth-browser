//! Game+8BA0's independent first-acquisition history, copied at Saved+58.
//! Item counts and seed/RNG stores cannot reconstruct this owner.
use serde::{Deserialize, Serialize};
use std::collections::{btree_map::Entry, BTreeMap};

/// Meaningful non-key operands of 576B40. Its +C field is one byte;
/// padding +D..F carries no owned value. Pool +488 is separate from its RNG.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcquisitionMetadata {
    pub source_update_word: u32,
    pub source_stage_word: u32,
    pub source_stage_type: u8,
    pub source_room_type_word: u32,
    pub source_pool_word_488: u32,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcquisitionHistory {
    /// 576CE0 compares only the signed ID at record+4. A present key with
    /// unknown first metadata stays present; duplicates cannot reconstruct it.
    pub first_records: BTreeMap<i32, Option<AcquisitionMetadata>>,
}

impl AcquisitionHistory {
    /// Read-only 576AF0 membership, independently of the retained metadata.
    pub fn has_seen(&self, item_id: i32) -> bool {
        self.first_records.contains_key(&item_id)
    }

    /// 576BB0 returns already-seen, the inverse of insertion success.
    /// Duplicate effects and owned-count removal never rewrite this record.
    pub fn seen_or_insert(
        &mut self,
        item_id: i32,
        available_source_record: Option<AcquisitionMetadata>,
    ) -> bool {
        match self.first_records.entry(item_id) {
            Entry::Occupied(_) => true,
            Entry::Vacant(entry) => {
                entry.insert(available_source_record);
                false
            }
        }
    }
}
