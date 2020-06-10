use crate::raft::storage::Storage;
use thiserror::Error;

// NONE is a placeholder node ID used when there is no leader.
pub const NONE: u64 = 0;

// State type represents the role of a node in a cluster.
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
struct Config<T: Storage> {
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
    storage: T,

    // Applied is the last applied index. It should only be set when restarting
    // raft. raft will not return entries to the application smaller or equal to
    // Applied. If Applied is unset when restarting, raft might return previous
    // applied entries. This is a very application dependent configuration.
    applied: u64,
}

impl<T: Storage> Config<T> {
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
pub struct Progress {
    pub _match: u64,
    pub next: u64,
}

pub struct Raft {

}
