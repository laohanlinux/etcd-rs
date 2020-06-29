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
use crate::raft::storage::StorageError;
use crate::raft::raftpb::raft::{Entry, Snapshot};
use thiserror::Error;
use crate::raft::unstable::Unstable;
use crate::raft::raft::NO_LIMIT;
use std::fmt::{Display, Formatter};
use crate::raft::util::limit_size;

#[derive(Error, Debug, PartialEq)]
pub enum RaftLogError {
    FromStorage(StorageError),
}


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

    // unstable contains all unstable entries and snapshot
    // they will be saved into storage
    unstable: Unstable,

    // committed is the highest log position that is known to be in
    // stable storage on a quorum of nodes
    committed: u64,

    // applied is the highest log position that the application has
    // been instructed to apply to its state machine.
    // Invariant: applied <= committed
    applied: u64,

    // log entries with index <= stabled are persisted to storage.
    // It is used to record the logs that are not persisted by storage yet.
    // Everytime handling `Ready`, the unstable logs will be included.
    stabled: u64,

    // all entries that have not yet compact.
    entries: Vec<Entry>,

    // max_next_ents_size is the maximum number aggregate byte size of the messages
    // returned from calls to nextEnts
    max_next_ents_size: u64,
}

impl<T: Storage> RaftLog<T> {
    // newLog returns log using the given storage. It recovers the log
    // to the state that it just commits and applies the latest snapshot.
    pub fn new(storage: T) -> Self {
        Self::new_log_with_size(storage, NO_LIMIT)
    }

    pub fn new_log_with_size(storage: T, max_next_ents_size: u64) -> Self {
        let mut log = Self {
            storage,
            committed: 0,
            applied: 0,
            stabled: 0,
            entries: vec![],
            max_next_ents_size,
        };
        let first_index = log.storage.first_index().unwrap();
        let last_index = log.storage.last_index().unwrap();
        // TODO
        log.unstable.offset = last_index + 1;
        log.committed = first_index - 1;
        log.applied = first_index - 1;
        log
    }

    // maybe_append returns `None` if the entries cannot be appended. Otherwise,
    // it returns `Some(last index of new entries)`
    pub(crate) fn maybe_append(&mut self, index: u64, log_term: u64, committed: u64, ents: &[Entry])
                               -> Option<u64> {}

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

    pub fn first_index(&self) -> u64 {
        if let Some(i) = self.unstable.maybe_first_index() {
            return i;
        }
        self.storage.first_index().unwrap()
    }

    // LastIndex returns the last index of the log entries
    pub fn last_index(&self) -> Result<u64, RaftLogError> {
        // Your Code Here (2A).
        Ok(0)
    }

    // pub fn entries(&self, i: u64, max_size: u64) -> Result<Vec<Entry>, RaftLogError> {

    pub(crate) fn commit_to(&mut self, to_commit: u64) {
        // never decrease commit
        if self.committed < to_commit {
            if self.last_index().unwrap() < to_commit {
                panic!("to_commit([]) is out of range [last_index({})]. Was the raft log corrupted, truncated, or lost?", to_commit, self.last_index().unwrap());
            }
            self.committed = to_commit;
        }
    }

    pub(crate) fn applied_to(&mut self, i: u64) {
        if i == 0 { return; }
        if self.committed < i || i < self.applied {
            panic!("applied({}) is out of range [prev_applied({}), committed({})]", i, self.applied, self.committed);
        }
        self.applied = i;
    }

    pub(crate) fn stable_to(&mut self, i: u64, t: u64) { self.unstable.stable_to(i, t); }

    pub(crate) fn stable_snap_to(&mut self, i: u64) { self.unstable.stable_snap_to(i) }

    pub(crate) fn last_term(&self) -> u64 {
        match self.term(self.last_index().unwrap()) {
            Ok(t) => t,
            Err(err) => panic!("unexpected error when getting the last term ({})", err)
        }
    }

    // Term return the term of the entry in the given index.
    pub fn term(&self, i: u64) -> Result<u64, RaftLogError> {
        // the valid from range is [index of dummy entry, last index]
        let dummy_index = self.first_index().unwrap() - 1;
        if i < dummy_index || i > self.last_index().unwrap() {
            // TODO: return an error instead?
            return Ok(0);
        }
        if let Some(t) = self.unstable.maybe_term(i) {
            return Ok(t);
        }

        self.storage.term(i).map_err(|e| {
            match e {
                StorageError::Compacted | StorageError::Unavailable => {}
                _ => panic!("unexpected error: {:?}", e)
            }
            RaftLogError::FromStorage(e)
        })
    }

    pub(crate) fn entries(&self, i: u64, max_size: u64) -> Result<Vec<Entry>, RaftLogError> {
        if i > self.last_index().unwrap() {
            return Ok(vec![]);
        }
        self.slice(i, self.last_index().unwrap(), max_size)
    }

    pub(crate) fn all_entries(&self) -> Vec<Entry> {
        match self.entries(self.first_index(), NO_LIMIT) {
            Ok(entries) => entries
            Err(RaftLogError::FromStorage(StorageError::Compacted)) => self.all_entries(), // try again if there was a racing compact
            Err(err) => panic!("{}", err) // TODO (xiangli): handle error?
        }
    }

    // is_up_to_date determines if the given (last_index, term) log is more up_to_date
    // by comparing the index and term of the last entries in the existing logs.
    // If the logs have last entries with different terms, then the log with the
    // later term is more up-to-date. If the logs end with the same term, then
    // whichever log has the larger last_index is more up-to-date. If the logs are
    // the same, the given log is up-to-date
    pub(crate) fn is_up_to_date(&self, lasti: u64, term: u64) -> bool {
        term > self.last_term() || (term == self.last_term() && lasti >= self.last_index().unwrap())
    }

    pub(crate) fn match_term(&self, i: u64, term: u64) -> bool {
        self.term(i).map(|t| t == term).map_err(|_| false).unwrap()
    }

    pub(crate) fn restore(&mut self, s: Snapshot) {
        info!("log [{}] starts to restore snapshot [index:{}, term: {}]", self, s.get_metadata().get_index(), s.get_metadata().get_term());
        self.committed = s.get_metadata().get_index();
        self.unstable.restore(s);
    }

    fn slice(&self, lo: u64, hi: u64, max_size: u64) -> Result<Vec<Entry>, RaftLogError> {
        self.must_check_out_of_bounds(lo, hi).map(|| vec![])?;
        if lo == hi {
            return Ok(vec![]);
        }
        let mut ents = Vec::new();
        if lo < self.unstable.offset {
            match self.storage.entries(lo, hi.min(self.unstable.offset), max_size) {
                Ok(entries) => {
                    // check if ents has reached the size limitation
                    if (entries.len() as u64) < hi.min(self.unstable.offset) - lo {
                        return Ok(entries);
                    }
                    ents = entries;
                }
                Err(StorageError::Compacted) => { return Ok(vec![]); }
                Err(StorageError::Unavailable) => panic!("entries[{}:{}] is unavailable from storage", lo, hi.min(self.unstable.offset)),
                Err(e) => panic!("{:?}", err),
            }
        }
        if hi > self.unstable.offset {
            let mut unstable = self.unstable.slice(lo.max(self.unstable.offset), hi);
            ents.extend_from_slice(&unstable);
        }

        Ok(limit_size(ents, max_size))
    }

    // l.first_index <= lo <= hi <= l.first_index + l.entries.len()
    fn must_check_out_of_bounds(&self, lo: u64, hi: u64) -> Result<(), RaftLogError> {
        if lo > hi {
            panic!("invalid slice {} > {}", lo, hi);
        }
        let fi = self.first_index();
        if lo < fi {
            return Err(RaftLogError::FromStorage(StorageError::Compacted));
        }
        let length = self.last_index().unwrap() + 1 - fi;
        if lo < fi || hi > fi + length {
            panic!("slice[{}:{}] out of bound [{}:{}]", lo, hi, fi, self.last_index().unwrap());
        }
        Ok(())
    }

    // fn zero_term_on_err_compacted(&self, t: u64, err: RaftLogError) -> u64 {}
}

impl Display for RaftLog<T> where T: Storage {
    fn fmt(&self, f: &mut Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "committed={}, applied={}, unstable.offset={}, len(unstable.entries)={}", self.committed, self.applied, self.unstable.offset, self.unstable.entries.len())
    }
}