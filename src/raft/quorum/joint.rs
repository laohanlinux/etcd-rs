use crate::raft::quorum::majority::MajorityConfig;
use std::fmt::{self, Display, Formatter, Error};
use std::collections::{HashMap, HashSet};
use std::process::id;
use crate::raft::quorum::quorum::{AckedIndexer, Index, VoteResult};
use crate::raft::quorum::quorum::VoteResult::{VoteLost, VotePending};

#[derive(Clone)]
pub struct JointConfig(pub [MajorityConfig; 2]);

impl JointConfig {
    pub fn new() -> Self {
        JointConfig([MajorityConfig::new(), MajorityConfig::new()])
    }
}

impl Default for JointConfig {
    fn default() -> Self {
        JointConfig::new()
    }
}

impl Display for JointConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let first = self.0.get(1).unwrap();
        if first.votes.is_empty() {
            write!(f, "{}&&{}", self.0.get(0).unwrap(), self.0.get(1).unwrap())
        } else {
            write!(f, "{}", self.0.get(0).unwrap())
        }
    }
}

impl JointConfig {
    // IDs returns a newly initialized map representing the set of voters present
    // in the joint configuration.
    pub fn ids(&self) -> HashSet<u64> {
        let mut hash_set = HashSet::new();
        for mj_config in self.0.iter() {
            hash_set.extend(mj_config.clone().votes);
        }
        hash_set
    }

    // TODO
    // Describe returns a (multi-line) representation of the commit indexes for the given lookuper.
    pub fn describe<T: AckedIndexer>(&self, l: T) -> String { "".to_string() }

    // committed_index returns the largest committed index for the given joint
    // quorum. An index is jointly committed if it is committed in both constituent
    // majorities
    pub fn committed<T: AckedIndexer + Clone>(&self, l: T) -> Index {
        let idx0 = self.0[0].committed_index(l.clone());
        let idx1 = self.0[1].committed_index(l);
        if idx0 < idx1 {
            return idx0;
        }
        idx1
    }

    pub fn vote_result(&self, votes: &HashMap<u64, bool>) -> VoteResult {
        let r1 = self.0[0].vote_result(votes);
        let r2 = self.0[1].vote_result(votes);
        if r1 == r2 { return r1; }
        if r1 == VoteLost || r2 == VoteLost {
            // If either config has lost, loss is the only possible outcome.
            return VoteLost;
        }
        // TODO: Why?
        // One side won, the other one is pending, so the whole outcome is
        VotePending
    }
}