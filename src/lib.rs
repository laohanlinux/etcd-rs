#![feature(is_sorted)]

use std::collections::HashMap;

pub mod raft;
pub mod mvcc;

pub type Index = u64;

pub fn string(index: Index) -> String {
    if index == u64::max_value() {
        "∞".to_string()
    }else {
        format!("{}", index)
    }
}

pub trait AckedIndexer {
    fn AckedIndex(&self, voter_id: &u64) -> Option<&Index>;
}

struct mapAckIndexer(HashMap<u64, Index>);

impl AckedIndexer for mapAckIndexer {
    fn AckedIndex(&self, voter_id: &u64) -> Option<&Index> {
        self.0.get(voter_id)
    }
}

#[derive(Debug, Clone)]
pub enum VoteResult {
    // VotePending indicates that the decision of the vote depends on future
    // votes, i.e. neither "yes" or "no" has reached quorum yet.
    VotePending,
    // VoteLost indicates that the quorum has votes "no"
    VoteLost,
    // VoteWon indicates that the quorum has voted "yes"
    VoteWon,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
