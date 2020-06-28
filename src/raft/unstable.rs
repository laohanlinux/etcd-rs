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
// might need to truncate the log before persisting unstable.entries.
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

    pub(crate) fn truncate_and_append(&mut self, ents: &[Entry]) {
        match ents[0].get_Index() {
            after if after == self.offset + self.entries.len() as u64 => {
                // after is the next index in the self.entries
                // directly append
                self.entries.extend_from_slice(ents);
            }
            after if after <= self.offset => {
                info!("replace the unstable entries from index {}", after);
                // The log is being truncated to before our current offset
                // portion, so set the offset and replace the entries
                self.offset = after;
                self.entries.clear();
                self.entries.extend_from_slice(ents);
            }
            after => {
                // truncate to after and copy to self.entries
                // then append
                info!("truncate the unstable entries before index {}", after);
                self.entries.truncate((after - self.offset) as usize);
                self.entries.extend_from_slice(&ents);
            }
        }
    }

    fn slice(&self, lo: u64, hi: u64) -> Vec<Entry> {
        self.must_check_out_of_bounds(lo, hi);
        self.entries[(lo - self.offset) as usize..(hi - self.offset) as usize].to_vec()
    }

    // self.offset <= lo <= hi <= self.offset + self.entries.len()
    fn must_check_out_of_bounds(&self, lo: u64, hi: u64) {
        if lo > hi {
            panic!("invalid unstable.slice {} > {}", lo, hi);
        }
        let upper = self.offset + self.entries.len() as u64;
        if lo < self.offset || hi > upper {
            panic!("unstable.slice[{}, {}] out of bound [{}, {}]", lo, hi, self.offset, upper);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::raft::raftpb::raft::{Snapshot, Entry};
    use crate::raft::unstable::Unstable;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn it_unstable_maybe_first_index() {
        // (entries, offset, snapshot, w_ok, w_index)
        let tests = vec![
            // no snapshot
            (vec![new_entry(5, 1)], 0, None, false, 0), (vec![], 0, None, false, 0),
            // has snapshot
            (vec![new_entry(5, 1)], 5, Some(new_snapshot(4, 1)), true, 5), (vec![], 5, Some(new_snapshot(4, 1)), true, 5)];
        for (i, (entries, offset, snapshot, w_ok, w_index)) in tests.iter().enumerate() {
            let mut u = Unstable {
                snapshot: snapshot.clone(),
                entries: entries.clone(),
                offset: *offset,
            };
            match u.maybe_first_index() {
                Some(i) => {
                    assert_eq!(i, *w_index);
                }
                None => assert!(!*w_ok),
            }
        }
    }

    #[test]
    fn it_maybe_last_index() {
        // (entries, offset, snapshot, w_ok, w_index)
        let tests = vec![
            // last in entries
            (vec![new_entry(5, 1)], 5, None, true, 5),
        ];
        for (i, (entries, offset, snapshot, w_ok, w_index)) in tests.iter().enumerate() {
            let mut u = Unstable {
                snapshot: snapshot.clone(),
                entries: entries.clone(),
                offset: *offset,
            };
            match u.maybe_last_index() {
                Some(i) => {
                    assert_eq!(i, *w_index);
                }
                None => assert!(!*w_ok),
            }
        }
    }

    fn new_entry(index: u64, term: u64) -> Entry {
        let mut entry = Entry::new();
        entry.set_Term(term);
        entry.set_Index(index);
        entry
    }

    fn new_snapshot(index: u64, term: u64) -> Snapshot {
        let mut snapshot = Snapshot::new();
        snapshot.mut_metadata().set_index(index);
        snapshot.mut_metadata().set_term(term);
        snapshot
    }
}