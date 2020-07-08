use crate::raft::storage::Storage;
use thiserror::Error;
use crate::raft::log::RaftLog;
use std::collections::HashMap;
use crate::raft::raftpb::raft::Message;
use crate::raft::raftpb::raft::MessageType::{MsgVoteResp, MsgPreVote};
use crate::raft::read_only::{ReadState, ReadOnly};
use crate::raft::tracker::ProgressTracker;

// NONE is a placeholder node ID used when there is no leader.
pub const NONE: u64 = 0;
pub const NO_LIMIT: u64 = u64::MAX;

// State type represents the role of a node in a cluster.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReadOnlyOption {
    // ReadOnlySafe guarantees the linearizability of the read only request by
    // communicating which the quorum. It is the default and suggested option.
    ReadOnlySafe,
    // ReadOnlyLeaseBased ensures linearizability of the read only request by
    // relying on the leader lease. It can be affected by clock drift.
    // If the clock drift is unbounded, leader might keep the lease longer than it
    // should (clock can move backward/pause without any bound). ReadIndex is not safe
    // in that case.
    ReadOnlyLeaseBased,
}


#[derive(Error, Debug, PartialEq)]
pub enum RafeError {
    // ErrProposalDropped is returned when the proposal is ignored by some cases,
    // so that the proposer can be notified and fail fast.
    #[error("raft proposal dropped")]
    ProposalDropped,
}

// Config contains the parameters to start a raft.
pub struct Config<S: Storage + Clone> {
    // id is the identify of the local raft. id cannot be 0;
    id: u64,

    // peers contains the IDs of all nodes (including self) in the raft cluster. It
    // should only be set when starting a new raft cluster. Restarting raft from
    // previous configuration will panic if peers is set. peer is private and only
    // used for testing right now.
    peers: Vec<u64>,

    // learners contains the IDs of all learner nodes (including self if the
    // local node is a learner) in the raft cluster. learners only receives
    // entries from the leader node. It does not vote or promote itself.
    learners: Vec<u64>,

    // ElectionTick is the number of Node.Tick invocations that must pass between
    // elections. That is, if a follower does not receive any message from the
    // leader of current term before ElectionTick has elapsed, it will become
    // candidate and start an election. ElectionTick must be greater than
    // HeartbeatTick. We suggest ElectionTick = 10 * HeartbeatTick to avoid
    // unnecessary leader switching.
    election_tick: isize,

    // HeartbeatTick is the number of Node.Tick invocations that must pass between
    // heartbeats. That is, a leader sends heartbeat messages to maintain its
    // leadership every HeartbeatTick ticks.
    heartbeat_tick: isize,

    // Storage is the storage for raft. raft generates entries and states to be
    // stored in storage. raft reads the persisted entries and states out of
    // Storage when it needs. raft reads out the previous state and configuration
    // out of storage when restarting.
    pub storage: S,

    // Applied is the last applied index. It should only be set when restarting
    // raft. raft will not return entries to the application smaller or equal to
    // Applied. If Applied is unset when restarting, raft might return previous
    // applied entries. This is a very application dependent configuration.
    pub applied: u64,

    // max_size_per_msg limits the max the byte size of each append message. Smaller
    // value lowers the raft recovery cost(initial probing and message lost
    // during normal operation). On the other side. it might affect the
    // throughput during normal replication. Note: math.MaxU64 for unlimited,
    // 0 from at most one entry per message.
    pub max_size_per_msg: u64,

    // max_committed_size_per_ready limits the size of the committed enables which
    // can be applied
    max_committed_size_per_ready: u64,

    // max_uncommitted_entries_size limits the aggregate byte size of the
    // uncommitted entries that may be appended to a leader's log. Once this
    // limit is exceeded, proposals will begin to return ErrProposalDropped
    // errors. Note: 0 for no limit.
    pub max_uncommitted_entries_size: u64,

    // max_inflight_msgs limits the max number of in-flight append messages during
    // optimistic replication phase. The application transportation layer usually
    // has its own sending buffer over TCP/UDP. Setting max_inflight_msgs to avoid
    // overflowing that sending buffer. TODO (xiangli): feedback to application to
    // limit the proposal rate?
    pub max_inflight_msgs: u64,

    // check_quorum specifies if the leader should check quorum activity. Leader
    // steps down when quorum is not active for an election_timeout.
    pub check_quorum: bool,

    // pre_vote enables the pre_vote algorithm described in raft thesis section
    // 9.6. This prevents disruption when a node that has been partitioned away
    // rejoins the cluster.
    pub pre_vote: bool,

    // ReadOnlyOption specifies how the read only request is processed.
    //
    // ReadOnlySafe guarantees the linearizability of the read only request by
    // communicating with the quorum. It is the default and suggested option.
    //
    // ReadOnlyLeaseBased ensures linearizability of the read only request by
    // replying on the leader lease. It can be affected by clock drift.
    // If the lock drift is unbounded, leader might keep the lease longer than it
    // should (clock and move backward/pause without any bound). ReadIndex is not safe
    // in that case.
    // check_quorum MUST be enables if ReadOnlyOption is ReadOnlyLeaseBased.
    pub read_only_option: ReadOnlyOption,

    // disable_proposal_forwarding set to true means that followers will drop
    // proposals, rather than forwarding them to the leader. One use case for
    // this feature would be in a situation where the Raft leader is used to
    // compute the data of a proposal, for example, adding a timestamp from a
    // hybrid logical clock to data in a monotonically increasing way. Forwarding
    // should be disable to prevent a follower with an inaccurate hybrid
    // logical clock from assigning the timestamp and then forwarding the data
    // to the leader.
    pub disable_proposal_forwarding: bool,
}


impl<S: Storage + Clone> Config<S> {
    pub fn validate(&mut self) -> Result<(), String> {
        if self.id == NONE {
            return Err("cannot be none as id".to_string());
        }
        if self.heartbeat_tick <= 0 {
            return Err("heartbeat tick must be greater than 0".to_string());
        }
        if self.election_tick <= 0 {
            return Err("election tick must be greater than 0".to_string());
        }

        if self.max_uncommitted_entries_size == 0 {
            self.max_uncommitted_entries_size = NO_LIMIT;
        }

        // default max_committed_size_per_ready to max_size_per_msg because they were
        // previously the same parameters.
        if self.max_committed_size_per_ready == 0 {
            self.max_committed_size_per_ready = self.max_size_per_msg;
        }

        if self.max_inflight_msgs <= 0 {
            return Err("max inflight messages must be greater than 0".to_string());
        }

        if self.read_only_option == ReadOnlyOption::ReadOnlyLeaseBased && !self.check_quorum {
            return Err("check_quorum must be enabled when ReadOnlyOption is ReadOnlyLeaseBased".to_string());
        }

        Ok(())
    }
}

// Progress represents a follower's progress in the view of the leader. Leader maintains
// progress of all followers, and sends entries to the follower based on its progress.
#[derive(Clone)]
pub struct Progress {
    pub _match: u64,
    pub next: u64,
}

pub struct Raft<S: Storage + Clone> {
    pub(crate) id: u64,
    pub term: u64,
    pub vote: u64,
    pub(crate) read_state: Vec<ReadState>,

    // the log
    pub raft_log: RaftLog<S>,

    max_msg_size: u64,
    max_uncommitted_size: u64,
    // TODO(tbg): rename to trk.
    // log replication progress of each peers.
    pub prs: ProgressTracker,
    // this peer's role
    pub state: State,
    // is_leader is true if the local raft node is a leaner
    is_learner: bool,

    // msgs need to send
    msgs: Vec<Message>,

    // the leader id
    pub lead: u64,
    // leadTransferee is id of the leader transfer target when its value is not zero.
    // Follow the procedure defined in section 3.10 of Raft phd thesis.
    // (https://web.stanford.edu/~ouster/cgi-bin/papers/OngaroPhD.pdf)
    // (Used in 3A leader transfer)
    lead_transferee: u64,
    // Only one config change may be pending (in the log, but not yet
    // applied) at a time. This is enforced via PendingConfIndex, which
    // is set to a value >= the log index of the latest pending
    // configuration change (if any). Config changes are only allowed to
    // be proposed if the leader's applied index is greater than this
    // value.
    // (Used in 3A conf change)
    pub pending_config_index: u64,
    // an estimate of the size of the uncommitted tail of the Raft log. Used to
    // prevent unbounded log growth. Only maintained by the leader. Reset on
    // term changes.
    uncommitted_size: u64,

    read_only: ReadOnly,

    // number of ticks since it reached last election_timeout
    election_elapsed: isize,

    // number of ticks sine it reached last heartbeat_timeout.
    // only leader keeps heartbeat_elapsed.
    heartbeat_elapsed: isize,

    check_quorum: bool,
    pre_vote: bool,

    // heartbeat interval, should send
    heartbeat_timeout: isize,
    // baseline of election interval
    election_timeout: isize,
    // randomized_election_timeout is a random number between
    // [election_timeout, 2*election_timeout-1]. It gets reset
    // when raft changes its state to follower or candidate.
    randomized_election_timeout: usize,
    disable_proposal_forwarding: bool,

    // votes records
    pub votes: HashMap<u64, bool>,
}

impl<S: Storage + Clone> Raft<S> {
    // TODO:
    pub fn new(mut config: Config<S>) {
        assert!(config.validate().is_ok());
        let mut raft_log = RaftLog::new_log_with_size(config.storage.clone(), config.max_committed_size_per_ready);
        let state_ret = config.storage.initial_state();
        assert!(state_ret.is_ok()); // TODO(bdarnell)
        let (mut hs, mut cs) = state_ret.unwrap();
        if !config.peers.is_empty() || !config.learners.is_empty() {
            // TODO(bdarnell): the peers argument is always nil except in
            // tests; the argument should be removed and these tests should be
            // updated to specify their nodes through a snapshot.
            panic!("cannot specify both new_raft(pees, learners) and ConfigState.(Voters, Learners)");
            cs.set_voters(config.peers.clone());
            cs.set_learners(config.learners.clone());
        }

        let mut raft = Raft{
            id: config.id,
            term: 0,
            vote: 0,
            read_state: vec![],
            raft_log,
            max_msg_size: config.max_size_per_msg,
            max_uncommitted_size: config.max_uncommitted_entries_size,
            prs: ProgressTracker::new(config.max_inflight_msgs),
            state: State::Follower,
            is_learner: false,
            msgs: vec![],
            lead: NONE,
            lead_transferee: 0,
            pending_config_index: 0,
            uncommitted_size: 0,
            read_only: ReadOnly {
                option: ReadOnlyOption::ReadOnlySafe,
                pending_read_index: Default::default(),
                read_index_queue: vec![]
            },
            election_elapsed: 0,
            heartbeat_elapsed: 0,
            check_quorum: false,
            pre_vote: false,
            heartbeat_timeout: 0,
            election_timeout: 0,
            randomized_election_timeout: 0,
            disable_proposal_forwarding: false,
            votes: Default::default()
        };

    }
    // send_append sends an append RPC with new entries (if any) and the
    // current commit index to the given peer. Returns true if a message was sent.
    fn send_append(&self, to: usize) -> bool {
        // Your Code Here (2A).
        false
    }

    // send_heartbeat sends a heartbeat RPC to the given peer.
    fn send_heartbeat(&mut self, to: u64) {
        // Your Code Here (2A).
    }

    // tick advances the interval logical clock by a single tick.
    pub(crate) fn tick(&mut self) {
        // Your Code Here (2A).
    }

    // become_follower transform this peer's state to follower
    pub(crate) fn become_follower(&mut self, term: u64, lead: u64) {
        // Your Code Here (2A).
    }

    // become_candidate transform this peer's state to candidate
    fn become_candidate(&mut self) {
        // Your Code Here (2A).
    }

    // become_leader transform this peer's state to leader
    fn become_leader(&mut self) {
        // Your Code Here (2A).
        // NOTE: Leader should be propose a noop entry on its term.
    }

    // Step the entrance of handle message, see `MessageType`
    // on `eraft.proto`. for what msgs should be handled
    pub fn step(&mut self, m: Message) -> Result<(), String> {
        // Your Code Here (2A).
        let term = m.get_term();
        if term == 0 {
            // local message
        } else if term > self.term {
            if m.get_field_type() == MsgVoteResp || m.get_field_type() == MsgPreVote {}
        }
        match self.state {
            State::Leader => {}
            State::Follower => {}
            State::Candidate => {}
        }
        Ok(())
    }

    // handle_append_entries handle append entries RPC request
    fn handle_append_entries(&mut self, m: Message) {
        // Your Code Here (2A).
    }

    // handle_heartbeat handle heartbeat RPC request
    fn handle_heartbeat(&mut self, m: Message) {
        // Your Code Here (2A).
    }

    // handle_snapshot handle Snapshot RPC request
    fn handle_snapshot(&mut self, m: Message) {
        // Your Code Here (2A).
    }

    // add_node add a new node to raft group
    fn add_node(&mut self, id: u64) {
        // Your Code Here (2A).
    }

    // remove_node remove a node from raft group
    fn remove_node(&mut self, id: u64) {
        // Your Code Here (2A).
    }
}
