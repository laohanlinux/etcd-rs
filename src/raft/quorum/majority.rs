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
