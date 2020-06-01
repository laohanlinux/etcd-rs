pub struct RangeOptions {
    limit: i64,
    pub rev: i64,
    pub count: bool,
}

pub struct RangeResult {
    // kvs: Vec<>
    pub rev: i64,
    pub count: isize,
}

pub trait ReadView {
    // FirstRev returns the first KV revision at the time of opending the txn.
    // After a compaction, the first revision increases to the compaction
    // revision.
    fn first_rev() -> i64;

    // rev returns the revision of the KV at the time of opending th txn.
    fn rev() -> i64;

    // Range gets the keys in the range at rangeRev.
    // The returned rev is the current revision of the KV when the operation is executed.
    // If rangeRev <=0, range gets the keys at currentRev.
    // If `end` is nil, the request returns the key.
    // If `end` is not nil and not empty, it gets keys in range [key, range_end]
    // If `end` is not nil and empty, it gets the keys greater than or equal to key.
    // Limit limits the number of keys returned.
    // If the required rev is compacted, ErrCompacted will be returned.
    fn range(key: Vec<u8>, end: Vec<u8>, ro: RangeOptions) -> Result<RangeResult, String>;
}

pub trait TxnRead: ReadView {
    // End marks the transaction is complete and ready to commit.
    fn end();
}

pub trait WriteView {
    // DeleteRange deletes the given range from the store.
    // A deleteRange increases the rev of the store if any key in the range exists.
    // The number of key deleted will be returned.
    // The returned rev is the current revision of the KV when the operation is executed.
    // It also generates one event for each key delete in the event history.
    // if the `end` is nil, deleteRange deletes the key.
    // if the `end` is not nil, deleteRange deletes the keys in range [key, range_end).
    fn delete_range(key: Vec<u8>, end: Vec<u8>) -> (i64, i64, i64);

    // Put puts the given key. value into the store. Put also takes additional argument lease to
    // attach a lease to a key-value pair as meta-data. KV implementation does not validate the lease
    // id.
    // A put also increases the rev of the store, and generates on event in the event history.
    // The returned rev is the current revision of the KV when the operation is executed.
    // Put(key: Vec<u8>, value: Vec<u8>)
}

// TxnWrite represents a transaction that can modify the store.
pub trait TxnWrite: TxnRead + WriteView {
    // Changes gets the changes made since opending the write txn
    // fn changes() changes
}

pub trait KV: ReadView + WriteView {
    // Read creates a read transaction
    // fn read()

    // Write creates a write transaction

    // Hash computes the hash of all MVCC revisions up to a given revision.

    // Compact frees all superseded keys with revisions less than rev.
}

pub trait WatchableKV: KV + Watchable {}

// Watchable is the trait that wraps the NewWatchStream function.
pub trait Watchable {
    // NewWatchStream returns a WatchStream that can be used to
    // watch events happened or happening on the KV.
    // fn NewWatchStream() -> WatchStream;
}


// ConsistentWatchableKV is WatchableKV that understands the consistency
// algorithm and consistent index.
// If the consistent index of executing entry is not larger than the
// consistent index of ConsistentWatchableKV, all operations in
// this entry are skipped and return empty response.
pub trait ConsistentWatchableKV: WatchableKV {
    // consistent_index returns the current consistent index of the KV
    fn consistent_index() -> u64;
}
