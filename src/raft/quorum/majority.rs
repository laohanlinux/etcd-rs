use std::collections::{HashSet, HashMap};
use std::fmt::{self, Formatter, Display, Write};
use std::process::id;
use std::cmp::Ordering;
use crate::raft::quorum::quorum::{AckedIndexer, Index, VoteResult};

#[derive(Clone)]
pub struct MajorityConfig {
    pub(crate) votes: HashSet<u64>,
}

impl MajorityConfig {
    // fn vote_result(votes: &HashMap<i64, bool>) -> Option<>
    pub fn new() -> Self {
        MajorityConfig { votes: HashSet::new() }
    }

    pub fn len(&self) -> usize {
        self.votes.len()
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
            if let Some(idx) = l.acked_index(vote) {
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

    // commit_index computes the committed index from those supplied via the
    // provide acked_index (for the active config).
    pub fn committed_index<T: AckedIndexer>(&self, l: T) -> Index {
        if self.votes.is_empty() {
            // This plays well with joint quorum which, when one of half is the zero
            // MajorityConfig, should behave like the other half.
            return u64::max_value();
        }
        // Use a on-stack slice to collect the committed indexes when n <= 7
        //
        0
    }

    // VoteResult takes a mapping of voters to yes/no (true/false) votes and returns
    // a result indicating whether the vote is pending (i.e. neither a quorum of
    // yes/no has been reached), won (a quorum of yes has been reached), or lost (a
    // quorum of no has been reached).
    pub fn vote_result(&self, votes: &HashMap<u64, bool>) -> VoteResult {
        if votes.is_empty() {
            // By convention, the elections on an empty config win. This comes in
            // handy with joint quorums because it'll make a half-populated joint
            // quorum behave like a majority quorum
            return VoteResult::VoteWon;
        }
        let mut ny: Vec<usize> = Vec::with_capacity(2); // vote counts for no and yes, responsibility
        let mut missing = 0;
        for id in &self.votes {
            match votes.get(id) {
                Some(v) => {
                    if *v {
                        ny[1] += 1;
                    } else {
                        ny[0] += 1;
                    }
                }
                None => {
                    missing += 1;
                }
            }
        }

        let q = self.votes.len() / 2 + 1;
        if ny[1] >= q {
            return VoteResult::VoteWon;
        }
        if ny[1] + missing >= q {
            return VoteResult::VotePending;
        }
        VoteResult::VoteLost
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
