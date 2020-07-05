use crate::raft::quorum::joint::JointConfig;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use crate::raft::quorum::majority::MajorityConfig;
use crate::raft::tracker::progress::{ProgressMap, Progress};
use crate::raft::raftpb::raft::ConfState;

pub mod inflights;
pub mod state;
pub mod progress;


// Config reflects the configuration tracked in a ProgressTacker.
#[derive(Default, Clone)]
pub struct Config {
    pub voters: JointConfig,
    // auto_leave is true if the configuration is joint and a transition to the
    // incoming configuration should be carried out automatically by Raft when
    // this is possible. If false, the configuration will be joint until the
    // application initiates than transition manually.
    pub auto_leave: bool,
    // Learner is a set of Ids corresponding to the learners active in th
    // current configutation.
    //
    // Invariant: Learners and Voters does not intersect, i.e if a peer is in
    // either half of the joint config, it can't be a learner; if it is a
    // learner it can't be in either half of the joint config. This invariant
    // simplifies the implementation since it allows peers to have clarity about
    // its current role without taking into account joint consensus.
    learners: HashSet<u64>,
    // When we return a voter into a learner during a joint consensus transition,
    // we cannot add the learner directly when entering the joint state. This is
    // because this would violate the invariant that the intersect of
    // voters and learners is empty. For example, assume a Voter is removed and
    // imediately re-added as a learner (or in other words, it it demoted):
    //
    // Initially, the configuration will be
    //
    //  voters: {1, 2, 3}
    //  learners: {}
    //
    // and we want to demote 3. Entering the joint configuration, we naively get
    //
    //  voters: {1, 2} & {1, 2, 3}
    //  learners: {3}
    //
    // but this violates invariant (3 is both voter and learner). Instead,
    // we get
    //
    //  voters: {1, 2} & {1, 2, 3}
    //  learners: {}
    //  next_learners: {3}
    //
    // Where 3 is not still purely a voter, but we are remembering the intention
    // to make it a learner upon transitioning into the final configuration:
    //
    //  voters: {1, 2}
    //  learners: {3}
    //  next_learners: {}
    //
    // Note that next_learners is not used while adding a learner that is not
    // also a voter in the joint config. In this case, the learner is added
    // right away when entering the joint configuration, so that it is caught up
    // as soon as possible.
    learners_next: HashSet<u64>,
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "voters={}", self.voters);
        if !self.learners.is_empty() {
            write!(f, " learners={}", MajorityConfig { votes: self.learners.clone() });
        }
        if !self.learners_next.is_empty() {
            write!(f, " learners_next={}", MajorityConfig { votes: self.learners_next.clone() });
        }
        if self.auto_leave {
            write!(f, " auto_leave");
        }
        Ok(())
    }
}

// ProgressTracker tracks the currently active configuration and the information
// known about the nodes and learners in it. In particular, it tracks the match
// index for each peer when in turn allows reasoning abound the committed index.
pub struct ProgressTracker {
    pub config: Config,
    pub progress: ProgressMap,
    pub votes: HashSet<u64>,
    pub max_inflight: isize,
}

impl ProgressTracker {
    pub fn new(max_inflight: isize) -> ProgressTracker {
        let mut p = ProgressTracker {
            config: Default::default(),
            progress: ProgressMap(HashMap::new()),
            votes: Default::default(),
            max_inflight,
        };
        p
    }

    // ConfState returns a ConfState representing the active configuration.
    pub fn config_state(&self) -> ConfState {
        let mut conf_state = ConfState::new();
        conf_state.set_voters(self.config.voters.0[0].as_slice());
        conf_state.set_voters_outgoing(self.config.voters.0[1].as_slice());
        conf_state.set_learners(self.config.learners.iter().map(|learner| *learner).collect());
        conf_state.set_learners_next(self.config.learners_next.iter().map(|learner| *learner).collect());
        conf_state.set_auto_leave(self.config.auto_leave);
        conf_state
    }
}