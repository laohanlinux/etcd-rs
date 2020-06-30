use crate::raft::raftpb::raft::{Entry, Snapshot};
use crate::raft::storage::{MemoryStorage, Storage};
use crate::raft::log::RaftLog;

pub(crate) fn new_entry(index: u64, term: u64) -> Entry {
    let mut entry = Entry::new();
    entry.set_Index(index);
    entry.set_Term(term);
    entry
}

pub(crate) fn new_entry_set(set: Vec<(u64, u64)>) -> Vec<Entry> {
    set.iter().map(|(index, term)| new_entry(*index, *term)).collect()
}

pub(crate) fn new_snapshot(index: u64, term: u64) -> Snapshot {
    let mut snapshot = Snapshot::new();
    snapshot.mut_metadata().set_index(index);
    snapshot.mut_metadata().set_term(term);
    snapshot
}

pub(crate) fn new_memory() -> MemoryStorage {
    let storage = MemoryStorage::new();
    storage
}

pub(crate) fn new_log() -> RaftLog<MemoryStorage> {
    let mut storage = new_memory();
    let mut log = RaftLog::new(storage);
    log
}

