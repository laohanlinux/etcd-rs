use crate::raft::raftpb::raft::{Entry, Snapshot};
use crate::raft::storage::{MemoryStorage, Storage};
use crate::raft::log::RaftLog;

pub fn new_entry(index: u64, term: u64) -> Entry {
    let mut entry = Entry::new();
    entry.set_Index(index);
    entry.set_Term(term);
    entry
}

pub fn new_entry_set(set: Vec<(u64, u64)>) -> Vec<Entry> {
    set.iter().map(|(index, term)| new_entry(*index, *term)).collect()
}

pub fn new_empty_entry_set() -> Vec<Entry> {
    Vec::new()
}

pub fn new_snapshot(index: u64, term: u64) -> Snapshot {
    let mut snapshot = Snapshot::new();
    snapshot.mut_metadata().set_index(index);
    snapshot.mut_metadata().set_term(term);
    snapshot
}

pub fn new_memory() -> MemoryStorage {
    let storage = MemoryStorage::new();
    storage
}

pub fn new_log() -> RaftLog<MemoryStorage> {
    let mut storage = new_memory();
    let mut log = RaftLog::new(storage);
    log
}

pub fn new_log_with_storage<T: Storage>(storage: T) -> RaftLog<T> {
    RaftLog::new(storage)
}