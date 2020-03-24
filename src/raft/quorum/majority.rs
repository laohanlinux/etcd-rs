use std::collections::{HashSet, HashMap};
use std::fmt::{self, Formatter, Display, Error, Write};

#[derive(Clone)]
pub struct MajorityConfig {
    votes: HashSet<i64>,
}

impl MajorityConfig {
    // fn vote_result(votes: &HashMap<i64, bool>) -> Option<>
}

impl From<String> for MajorityConfig {
    fn from(_: String) -> Self {
        unimplemented!()
    }
}

impl From<Vec<i64>> for MajorityConfig {
    fn from(v: Vec<i64>) -> Self {
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
        let mut votes: Vec<i64> = self.votes.iter().map(|v| *v).collect();
        votes.sort();
        let mut votes: Vec<String> = votes.iter().map(|v| format!("{}", v)).collect();
        let mut s: String = votes.iter()
            .map(|s| s.chars())
            .flatten()
            .collect();
        write!(f, "({})", s)
    }
}
