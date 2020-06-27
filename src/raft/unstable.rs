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
    pub(crate) fn maybe_first_index(&self) -> (u64, bool) {
        if let Some(ref snapshot) = self.snapshot {
            return (snapshot.get_metadata().get_index() + 1, true);
        }
        (0, false)
    }
}