use crate::raft::quorum::majority::MajorityConfig;
use std::fmt::{self, Display, Formatter, Error};
use std::collections::{HashMap, HashSet};
use std::process::id;
use crate::raft::quorum::quorum::AckedIndexer;

pub struct JointConfig([MajorityConfig; 2]);

impl JointConfig {
    pub fn new() -> Self {
        JointConfig([MajorityConfig::new(), MajorityConfig::new()])
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
    pub fn describe<T: AckedIndexer>(&self, l: T) -> String {"".to_string()}
}