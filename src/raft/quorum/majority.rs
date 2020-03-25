use std::collections::HashSet;
use std::fmt::{self, Formatter, Display, Write};

#[derive(Clone)]
pub struct MajorityConfig {
    pub(crate) votes: HashSet<u64>,
}

impl MajorityConfig {
    // fn vote_result(votes: &HashMap<i64, bool>) -> Option<>
    pub fn new() -> Self {
        MajorityConfig { votes: HashSet::new() }
    }
}

impl From<String> for MajorityConfig {
    fn from(_: String) -> Self {
        unimplemented!()
    }
}

impl From<Vec<u64>> for MajorityConfig {
    fn from(v: Vec<u64>) -> Self {
        let mut config = MajorityConfig {
            votes: HashSet::new(),
        };
        for item in v.iter() {
            config.votes.insert(*item);
        }
        config
    }
}

impl Display for MajorityConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut votes: Vec<u64> = self.votes.iter().map(|v| *v).collect();
        votes.sort();
        let votes: Vec<String> = votes.iter().map(|v| format!("{}", v)).collect();
        let s: String = votes.iter()
            .map(|s| s.chars())
            .flatten()
            .collect();
        write!(f, "({})", s)
    }
}
