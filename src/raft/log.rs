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


use crate::raft::storage::Storage;
use crate::raft::raftpb::raft::{Entry, Snapshot};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum RaftLogError {}


// RaftLog manage the log entries, its struct look like:
//
//  snapshot/first.....applied....committed....stabled.....last
//  --------|------------------------------------------------|
//                            log entries
//
// for simplify the RaftLog implement should manage all log entries
// that not truncated
pub struct RaftLog<T: Storage> {
    // storage contains all stable entries since the last snapshot
    storage: T,

    // committed is the highest log position that is known to be in
    // stable storage on a quorum of nodes
    committed: u64,

    // applied is the highest log position that the application has
    // been instructed to apply to its state machine.
    // Invariant: applied <= committed
    applied: u64,

    // log entries with index <= stabled are persisted to storage.
    // It is used to record the logs that are not persisted by storage yet.
    // Everytime handling `Ready`, the unstabled logs will be included.
    stabled: u64,

    // all entries that have not yet compact.
    entries: Vec<Entry>,

    // the incoming unstable snapshot, if any.
    // (Used in 2C)
    pending_snapshot: Option<Snapshot>,

    // Your Data Here (2A).
}

impl<T: Storage> RaftLog<T> {
    // newLog returns log using the given storage. It recovers the log
// to the state that it just commits and applies the latest snapshot.
    pub fn new(storage: T) -> Self {
        // Your Code Here (2A).
        RaftLog {
            storage,
            committed: 0,
            applied: 0,
            stabled: 0,
            entries: vec![],
            pending_snapshot: None,
        }
    }

    // We need to compact the log entries in some point of time like
// storage compact stabled log entries prevent the log entries
// grow unlimitedly in memory.
    pub fn maybe_compact(&self) -> bool {
        // Your Code Here (2C).
        false
    }

    // unstable_entries returns all the unstable entries
    pub fn unstable_entries(&self) -> Vec<Entry> {
        // Your Code Here (2A).
        vec![]
    }

    // nextEnts returns all the committed but not applied entries
    pub fn nextEnts(&mut self) -> Vec<Entry> {
        // Your Code Here (2A).
        vec![]
    }

    // LastIndex returns the last index of the log entries
    pub fn last_index(&self) -> Result<u64, RaftLogError> {
        // Your Code Here (2A).
        Ok(0)
    }

    // Term return the term of the entry in the given index.
    pub fn term(&self, i: u64) -> Result<u64, RaftLogError> {
        // Your Code Here (2A).
        Ok(0)
    }
}