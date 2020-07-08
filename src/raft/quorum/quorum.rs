use std::collections::HashMap;

pub type Index = u64;

pub fn string(index: Index) -> String {
    if index == u64::max_value() {
        "∞".to_string()
    } else {
        format!("{}", index)
    }
}

pub trait AckedIndexer {
    fn acked_index(&self, voter_id: &u64) -> Option<&Index>;
}

struct MapAckIndexer(HashMap<u64, Index>);

impl AckedIndexer for MapAckIndexer {
    fn acked_index(&self, voter_id: &u64) -> Option<&Index> {
        self.0.get(voter_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoteResult {
    // VotePending indicates that the decision of the vote depends on future
    // votes, i.e. neither "yes" or "no" has reached quorum yet.
    VotePending,
    // VoteLost indicates that the quorum has votes "no"
    VoteLost,
    // VoteWon indicates that the quorum has voted "yes"
    VoteWon,
}

// impl PartialEq for VoteResult {
//     fn eq(&self, other: &Self) -> bool {
//         match self {
//
//         }
//     }
// }