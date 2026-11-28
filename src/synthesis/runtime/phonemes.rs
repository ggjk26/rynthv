use super::Runtime;
use crate::synthesis::{plan::OwnerId, rendered::PhonemeSnapshot};
use std::collections::BTreeMap;

impl Runtime {
    /// Published phonemes are already checked against the synchronized plan.
    pub fn phonemes_current(&self, owner: OwnerId, revision: u64) -> Option<&PhonemeSnapshot> {
        if self.last_revision == Some(revision) {
            self.phonemes(owner)
        } else {
            None
        }
    }

    pub(super) fn sync_phonemes(&mut self) {
        let plan = self.planner.plan();
        self.phonemes.retain(|owner, snapshot| {
            plan.requests
                .get(owner)
                .is_some_and(|request| *request.input == *snapshot.input)
        });
    }

    pub(super) fn complete_phonemes(&mut self, snapshots: BTreeMap<OwnerId, PhonemeSnapshot>) {
        for (owner, snapshot) in snapshots {
            if self
                .planner
                .plan()
                .requests
                .get(&owner)
                .is_some_and(|request| *request.input == *snapshot.input)
            {
                self.phonemes.insert(owner, snapshot);
            }
        }
    }
}
