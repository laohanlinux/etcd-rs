use std::collections::HashSet;
use std::fmt::{self, Formatter, Display, Write};
use crate::{Index, AckedIndexer};
use std::process::id;
use std::cmp::Ordering;

#[derive(Clone)]
pub struct MajorityConfig {
    pub(crate) votes: HashSet<u64>,
}

impl MajorityConfig {
    // fn vote_result(votes: &HashMap<i64, bool>) -> Option<>
    pub fn new() -> Self {
        MajorityConfig { votes: HashSet::new() }
    }

    pub fn describe<T: AckedIndexer>(&self, l: T) -> String {
        if self.votes.is_empty() {
            return "<empty majority quorum>".to_string();
        }

        #[derive(Default)]
        struct Tup {
            id: u64,
            idx: Index,
            ok: bool,
            // idx found?
            bar: isize, // length of bar displayed for this up
        }

        // Below, populate .bar so that the i-th largest commit index has bar i (we
        // plot this as sort of a progress bar). The actual code is a bit more
        // complicated and also makes sure that equal index => equal bar.
        let n = self.votes.len();
        let mut info: Vec<Tup> = Vec::new();
        for vote in self.votes.iter() {
            if let Some(idx) = l.AckedIndex(vote) {
                info.push(Tup {
                    id: *vote,
                    idx: *idx,
                    ok: true,
                    bar: 0,
                });
            } else {
                info.push(Tup {
                    id: *vote,
                    idx: 0,
                    ok: false,
                    bar: 0,
                });
            }
        }

        // Sort by index
        info.sort_by(|a, b| {
            if a.idx == b.idx {
                a.id.cmp(&b.id)
            } else {
                a.idx.cmp(&b.idx)
            }
        });

        // Populate .bar.
        "".to_string()
    }

    pub fn as_slice(&self) -> Vec<u64> {
        let mut s1: Vec<u64> = self.votes.iter().map(|v| *v).collect();
        s1.sort_by_key(|v| *v);
        s1
    }
}

impl From<&Vec<u64>> for MajorityConfig {
    fn from(v: &Vec<u64>) -> Self {
        let mut config = MajorityConfig {
            votes: HashSet::new(),
        };
        for item in v.iter() {
            config.votes.insert(*item);
        }
        config
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
        let s: String = votes.join(",");
        write!(f, "({})", s)
    }
}

#[cfg(test)]
mod tests {
    use crate::raft::quorum::majority::MajorityConfig;

    #[test]
    fn t_majority() {
        let mut majority = MajorityConfig::new();
        majority.votes.insert(0);
        majority.votes.insert(1);
        assert_eq!("(0,1)", format!("{}", majority));
        let mut majority = MajorityConfig::new();
        assert_eq!("()", format!("{}", majority));

        let v = &vec![0, 1, 2];
        let majority: MajorityConfig = v.into();
        assert_eq!("(0,1,2)", format!("{}", majority));
        let majority: MajorityConfig = v.into();
        assert_eq!("(0,1,2)", format!("{}", majority));

        let mut majority = MajorityConfig::new();
        majority.votes.insert(0);
        assert_eq!(vec![0], majority.as_slice());
    }
}
