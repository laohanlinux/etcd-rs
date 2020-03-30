use crate::raft::tracker::state::StateType;
use crate::raft::tracker::inflights::Inflights;

// Progress represents a follower's progress in the view of the leader. Leader
// maintains progresses of all followers, and sends entries to the follower
// based on its progress.
//
// NB(tg): Progress is basically a state machine whose transactions are mostly
// strewn around `*raft.raft`. Additionally, some fields are only used when in a 
// certain State. All of this isn't ideal
pub struct Progress {
    _match: u64,
    next: u64,

    // State defines how the leader should interact with the follower.
    //
    // When in StateProbe, leader sends at most one replication message
    // per heartbeat interval. It also probes actual progress of the follower.
    //
    // When in StateReplicate, leader optimistically increase next
    // to the latest entry sent after sending replication message. This is
    // an optimized state for fast replicating log entries to the follower.
    //
    // When in StateSnapshot, leader should have sent out snapshot
    // before and stops sending any replication message.
    state: StateType,

    // PendingSnapshot is used in StateSnapshot.
    // If there is a pending snapshot, the pendingSnapshot will be set to the
    // index of the snapshot. If pendingSnapshot is set, the replication process of
    // this Progress will be paused. raft will not resend snapshot until the pending one
    // is reported to be failed.
    pending_snapshot: u64,

    // recent_active is true if the progress is recently active. Receiving any messages
    // from the corresponds follower indicates the progress is active.
    // recent_active can be reset to false after an election timeout.
    //
    // TODO(tbg): the leader should always have this set to true.
    recent_active: u64,

    // ProbeSent is used while this follower is in StateProbe. When ProbeSent is
    // true, raft should pause sending replication message to this peer until
    // ProbeSent is reset. See ProbeAcked() and IsPaused().
    probe_sent: bool,

    // Inflights is a sliding window for the inflight messages.
    // Each inflight message contains one or mre log entries.
    // The max number of entries per message is defined in raft config as MaxSizePerMsg.
    // Thus inflight effectively limits both the number of inflight messages
    // and the bandwidth each process can use.
    // When inflights is Full, no more message should be sent.
    // When a leader sends out a message, the index of the last
    // entry should be added to inflights. The index MUST be added
    // into inflights in order.
    // When a leader receives a reply, the previous inflights should
    // be freed by calling inflights.FreeLe with the index of the last
    // received entry.
    inflights: Inflights,

    // IsLeader is true if this progress is tracked for a leader.
    is_leader: bool,
}

impl Progress {
    // ResetState moves that Progress into the specified State, resetting ProbeSent,
    // PendingSnapshot, and inflight
    pub fn reset_state(&mut self, state: StateType) {
        self.probe_sent = false;
        self.pending_snapshot = 0;
        self.state = state;
        self.inflights.reset();
    }

    // probe_acked is called when this peer has accepted an append. It resets
    // probe_sent to signal that additional append messages should be sent without
    // further delay.
    pub fn probe_acked(&mut self) {
        self.probe_sent = false;
    }

    
}
