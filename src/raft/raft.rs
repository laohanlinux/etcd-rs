use crate::raft::storage::Storage;
use thiserror::Error;
use crate::raft::log::RaftLog;
use std::collections::HashMap;
use crate::raft::raftpb::raft::Message;
use crate::raft::raftpb::raft::MessageType::{MsgVoteResp, MsgPreVote};

// NONE is a placeholder node ID used when there is no leader.
pub const NONE: u64 = 0;

// State type represents the role of a node in a cluster.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Follower,
    Candidate,
    Leader,
}

#[derive(Error, Debug, PartialEq)]
pub enum RafeError {
    // ErrProposalDropped is returned when the proposal is ignored by some cases,
    // so that the proposer can be notified and fail fast.
    #[error("raft proposal dropped")]
    ProposalDropped,
}

// Config contains the parameters to start a raft.
pub struct Config<S: Storage> {
    // id is the identify of the local raft. id cannot be 0;
    id: u64,

    // peers contains the IDs of all nodes (including self) in the raft cluster. It
    // should only be set when starting a new raft cluster. Restarting raft from
    // previous configuration will panic if peers is set. peer is private and only
    // used for testing right now.
    peers: Vec<u64>,

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
    storage: S,

    // Applied is the last applied index. It should only be set when restarting
    // raft. raft will not return entries to the application smaller or equal to
    // Applied. If Applied is unset when restarting, raft might return previous
    // applied entries. This is a very application dependent configuration.
    applied: u64,
}

impl<S: Storage> Config<S> {
    pub fn validate(&self) -> Result<(), String> {
        if self.id == NONE {
            return Err("cannot be none as id".to_string());
        }
        if self.heartbeat_tick <= 0 {
            return Err("heartbeat tick must be greater than 0".to_string());
        }
        if self.election_tick <= 0 {
            return Err("election tick must be greater than 0".to_string());
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

pub struct Raft<S: Storage> {
    pub(crate) id: u64,
    pub term: u64,
    pub vote: u64,
    // the log
    pub raft_log: RaftLog<S>,
    // log replication progress of each peers.
    pub prs: HashMap<u64, Progress>,
    // this peer's role
    pub state: State,
    // votes records
    pub votes: HashMap<u64, bool>,
    // msgs need to send
    msgs: Vec<Message>,
    // the leader id
    pub lead: u64,
    // heartbeat interval, should send
    heartbeat_timeout: isize,
    // baseline of election interval
    election_timeout: isize,
    // number of ticks sine it reached last heartbeat_timeout.
    // only leader keeps heartbeat_elapsed.
    heartbeat_elapsed: isize,
    // number of ticks since it reached last election_timeout
    election_elapsed: isize,
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
}

impl<S: Storage> Raft<S> {
    // TODO:
    pub fn new(config: Config<S>) {
        assert!(config.validate().is_ok());
        // Your Code Here (2A).
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
            if m.get_field_type() == MsgVoteResp || m.get_field_type() == MsgPreVote {

            }
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
