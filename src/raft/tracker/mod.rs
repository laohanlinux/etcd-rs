use crate::raft::quorum::joint::JointConfig;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use crate::raft::quorum::majority::MajorityConfig;
use crate::raft::tracker::progress::{ProgressMap, Progress};
use crate::raft::raftpb::raft::ConfState;
use crate::raft::quorum::quorum::{Index, AckedIndexer, VoteResult};
use std::iter::Cloned;
use crate::raft::quorum::quorum::VoteResult::VoteWon;

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
    pub votes: HashMap<u64, bool>,
    pub max_inflight: u64,
}

impl Clone for ProgressTracker {
    fn clone(&self) -> Self {
        let mut to = ProgressTracker::new(self.max_inflight);
        to.config = self.config.clone();
        let mut progress_inner = HashMap::new();
        progress_inner.extend(self.progress.0.iter().map(|(key, value)| { (*key, value.clone()) }));
        to.progress = ProgressMap(progress_inner);
        to.votes = self.votes.clone();
        to
    }
}

impl ProgressTracker {
    pub fn new(max_inflight: u64) -> ProgressTracker {
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

    // is_singleton returns true if (and only if) there is only one voting number
    // (i.e. the leader) in the current configuration.
    pub fn is_singleton(&self) -> bool {
        self.config.voters.0[0].len() == 1 && self.config.voters.0[1].len() == 1
    }

    // committed returns the largest log index known to be committed based on what
    // the voting members of the group have acknowledged.
    pub fn committed(&mut self) -> u64 {
        self.config.voters.committed(MatchAckIndexer::from(&self.progress))
    }

    // visit invokes the supplied closure for all tracked progresses in stable order.
    pub fn visit<F>(&self, mut f: F) where F: FnMut(u64, &Progress) {
        let n = self.progress.0.len();
        // We need to sort the IDs and don't want to allocate since this is hot code.
        // The optimized here mirrors that in `(MajorityConfig).CommittedIndex`,
        // see there for details
        // TODO optimized
        let mut ids: Vec<u64> = Vec::new();
        ids.extend(self.progress.0.keys().into_iter());
        ids.sort_by_key(|k| *k);
        for id in ids {
            f(id, self.progress.0.get(&id).unwrap());
        }
    }

    // returns true if the quorum is active from the view of the local
    // raft state machine. Otherwise, it returns false.
    pub fn quorum_active(&self) -> bool {
        let mut votes = HashMap::new();
        self.visit(|id, progress| {
            if progress.is_leader {
                return; // TODO Why?
            }
            votes.insert(id, progress.recent_active);
        });
        self.config.voters.vote_result(&votes) == VoteWon
    }

    // returns a sorted slice of voters.
    pub fn voter_nodes(&self) -> Vec<u64> {
        let mut nodes: Vec<u64> = self.config.voters.ids().iter().map(|id| *id).collect();
        nodes.sort_by_key(|id| *id);
        nodes
    }

    // returns a sorted slice of voters
    pub fn learner_nodes(&self) -> Vec<u64> {
        let mut nodes: Vec<u64> = self.config.learners.iter().map(|id| *id).collect();
        nodes.sort_by_key(|id| *id);
        nodes
    }

    // prepares for a new round of vote counting via record_vote.
    pub fn reset_votes(&mut self) {
        self.votes.clear();
    }

    // records that the node with the given id voted for this Raft
    // instance if v == true (and declined it otherwise)
    pub fn record_vote(&mut self, id: u64, v: bool) {
        self.votes.entry(id).or_insert(v);
    }

    // returns the number of granted and rejected votes, and whether the election outcome is known
    pub fn tally_votes(&self) -> (isize, isize, VoteResult) {
        // Make sure to populate granted/rejected correctly even if the votes slice
        // contains members no larger part of the configuration. This doesn't really
        // matter in the way the numbers are used (they're information), but might
        // as well get it right.
        let mut granted = 0;
        let mut rejected = 0;
        for (id, progress) in &self.progress.0 {
            if progress.is_leader {
                continue;
            }
            match self.votes.get(id) {
                Some(v) => {
                    if *v {
                        granted += 1;
                    } else {
                        rejected += 1;
                    }
                }
                None => {}
            }
        }
        (granted, rejected, self.config.voters.vote_result(&self.votes))
    }
}

#[derive(Clone)]
pub(crate) struct MatchAckIndexer(HashMap<u64, Progress>);

impl MatchAckIndexer {
    pub(crate) fn new() -> Self {
        MatchAckIndexer(HashMap::new())
    }
}

impl AckedIndexer for MatchAckIndexer {
    fn acked_index(&self, voter_id: &u64) -> Option<&u64> {
        self.0.get(voter_id).map(|pr| &pr._match)
    }
}

impl From<&ProgressMap> for MatchAckIndexer {
    fn from(progress: &ProgressMap) -> Self {
        let mut match_ack_indexer = MatchAckIndexer(HashMap::new());
        match_ack_indexer.0.clone_from(&progress.0);
        match_ack_indexer
    }
}