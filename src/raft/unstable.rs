// Copyright 2015 The etcd Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::raft::raftpb::raft::{Snapshot, Entry};

// unstable.entries[i] has raft log position i+unstable.offset.
// Note that unstable.offset may be less than highest log
// position in storage; this means that the next write to storage
// might nned to truncate the log before persisting unstable.entries.
pub(crate) struct Unstable {
    // the incoming unstable snapshot, if any.
    snapshot: Option<Snapshot>,
    // all entries that have not been yet been written to storage.
    entries: Vec<Entry>,
    offset: u64,
}

impl Unstable {
    // maybe_first_index returns the index of the first possible entry in entries
    // if it has a snapshot.
    pub(crate) fn maybe_first_index(&self) -> Option<u64> {
        self.snapshot.as_ref().map(|snapshot| snapshot.get_metadata().get_index() + 1)
    }

    // maybe_last_index returns last index if it has at least one
    // unstable entry or snapshot
    pub(crate) fn maybe_last_index(&self) -> Option<u64> {
        if !self.entries.is_empty() {
            return Some(self.offset + self.entries.len() as u64 - 1);
        }
        self.snapshot.as_ref().map(|snapshot| snapshot.get_metadata().get_index())
    }

    // maybe_term returns the term of the entry at index i, if there
    // is any.
    pub(crate) fn maybe_term(&self, i: u64) -> Option<u64> {
        if i < self.offset {
            if let Some(snapshot) = self.snapshot.as_ref() {
                if snapshot.get_metadata().get_index() == i {
                    return Some(snapshot.get_metadata().get_term());
                }
            }
            return None;
        }
        match self.maybe_last_index() {
            Some(index) => {
                if i > index {
                    None
                } else {
                    Some(self.entries.get((i - self.offset) as usize).unwrap().get_Term())
                }
            }
            None => {
                None
            }
        }
    }

    pub(crate) fn stable_to(&mut self, i: u64, t: u64) {
        if let Some(gt) = self.maybe_term(i) {
            // if i < offset, term is matched with the snapshot
            // only update the unstable entries if term is matched with
            // an unstable entry
            if gt == t && i >= self.offset {
                let start = i + 1 - self.offset;
                // TODO: add
                self.entries.drain(..start as usize);
                self.offset = i + 1;
            }
        }
    }

    pub(crate) fn stable_snap_to(&mut self, i: u64) {
        if let Some(ref snapshot) = self.snapshot {
            if snapshot.get_metadata().get_index() == i {
                self.snapshot = None;
            }
        }
    }

    pub(crate) fn restore(&mut self, s: Snapshot) {
        self.offset = s.get_metadata().get_index() + 1;
        self.entries.clear();
        self.snapshot = Some(s);
    }

    pub(crate) fn truncate_and_append(&mut self, ents: Vec<Entry>) {
        match ents[0].get_Index() {
            after if after == self.offset + self.entries.len() as u64 => {
                // after is the next index in the self.entries
                // directly append
                self.entries.extend_from_slice(ents.as_slice());
            }
            after if after <= self.offset => {
                info!("replace the unstable entries from index {}", after);
                // The log is being truncated to before our current offset
                // portion, so set the offset and replace the entries
                self.offset = after;
                self.entries = ents;
            }
            after => {
                // truncate to after
                // TODO
            }
        }
    }
}