use crate::raft::tracker::state::StateType;
use crate::raft::tracker::inflights::Inflights;
use crate::raft::tracker::state::StateType::{StateProbe, StateReplicate};
use std::fmt::{self, Display, Formatter, Error};
use std::collections::HashMap;

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
    recent_active: bool,

    // ProbeSent is used while this follower is in StateProbe. When ProbeSent is
    // true, raft should pause sending replication message to this peer until
    // ProbeSent is reset. See ProbeAcked() and IsPaused().
    probe_sent: bool,

    // IsLeader is true if this progress is tracked for a leader.
    is_leader: bool,

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

    // BecomeProbe transaction into StateProbe. Next is reset to Match+1 or,
    // optionally and if larger, the index of the pending snapshot.
    pub fn become_probe(&mut self) {
        // If the original state is StateSnapshot, Progress knows that
        // the pending snapshot has been sent to this peer Successfully, then
        // probes from pendingSnapshot + 1.
        if self.state == StateType::StateSnapshot {
            let pending_snapshot = self.pending_snapshot;
            self.reset_state(StateType::StateProbe);
            self.next = (self._match + 1).max(pending_snapshot + 1);
        } else {
            self.reset_state(StateType::StateProbe);
            self.next = self._match + 1;
        }
    }

    // Become Replicate transaction into StateReplicate, resetting Next to _match + 1
    pub fn become_replicate(&mut self) {
        self.reset_state(StateType::StateSnapshot);
        self.next = self._match + 1;
    }

    // BecomeSnapshot moves that Progress to StateSnapshot with the specified pending
    // snapshot
    pub fn become_snapshot(&mut self, snapshot: u64) {
        self.reset_state(StateType::StateSnapshot);
        self.pending_snapshot = snapshot;
    }

    // maybe_update is called when an MsgAppResp arrives from the follower, with the
    // index acked by it. The method returns false if the given n index comese of from
    // an outdated message. Otherwise it updates the progress and return true.
    pub fn maybe_update(&mut self, n: u64) -> bool {
        let mut update = false;
        if self._match < n {
            self._match = n;
            update = true;
            self.probe_acked();
        }
        if self.next < n + 1 {
            self.next = n + 1;
        }
        update
    }

    // OptimisticUpdate signals the appends all the way up to and including index n
    // are in-flight. As a result. Next is increased to n + 1
    pub fn optimistic_update(&mut self, n: u64) {
        self.next = n + 1;
    }

    // MaybeDecrTo adjust the Progress to the receipt of a MsgApp rejection. The
    // arguments are the index the follower rejected to append to its log, and its
    // last index.
    //
    // rejecteds can happen spuriously as messages are sent out of order or
    // duplicated. In such cases, the rejection pertains to an index that the
    // Progress already knows were previously acknowledged, and false is returned
    // without changing the Progress.
    //
    // If the rejection is genuine, Next is lowered sensibly, and the Progress is
    // cleared for sending log entries

    pub fn maybe_decr_to(&mut self, rejected: u64, last: u64) -> bool {
        if self.state == StateReplicate {
            // The rejected must be stale if the progress has matched and "rejected"
            // is smaller than "match".
            if rejected <= self._match {
                return false;
            }
            // Directly decrease next to match + 1
            //
            // TODO(tbg): Why not use last if it's larger?
            self.next = self._match + 1;
            return true;
        }

        // The rejection must be stale if "rejected" does not match next - 1. This
        // is because non-replicating followers are probed one entry at a time.
        if self.next - 1 != rejected {
            return false;
        }

        self.next = rejected.min(last + 1);
        if self.next < 1 {
            self.next = 1;
        }
        self.probe_sent = false;
        true
    }

    // IsPaused return whether sending log entries to this node has been throttled.
    // This is done when a node has rejected recent MsgApp, is currently waiting
    // for a snapshot, or has reached the MaxInflightMsgs limit. In normal
    // operation, this is false. A throttled node will be contacted less frequently
    // until it has reached a state in which it's able to accept a steady stream of
    // log entries again.
    pub fn is_paused(&self) -> bool {
        match self.state {
            StateType::StateProbe => self.probe_sent,
            StateType::StateReplicate => self.inflights.full(),
            StateType::StateSnapshot => true
        }
    }
}

impl Display for Progress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} match={} next={}", self.state, self._match, self.next)?;
        if self.is_leader {
            write!(f, " learner")?;
        }
        if self.is_paused() {
            write!(f, " paused")?;
        }
        if self.pending_snapshot > 0 {
            write!(f, " pendingSnap={}", self.pending_snapshot)?;
        }
        if !self.recent_active {
            write!(f, " inactive")?;
        }
        let n = self.inflights.count();
        if n > 0 {
            write!(f, " inflight={}", n)?;
            if self.inflights.full() {
                write!(f, "[full]")?;
            }
        }
        Ok(())
    }
}

// ProgressMap is a map of *Progress
#[derive(Default)]
pub struct ProgressMap(pub HashMap<u64, Progress>);

impl Display for ProgressMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut keys: Vec<u64> = self.0.keys().map(|uid| *uid).collect();
        keys.sort_by_key(|k| *k);
        let keys: Vec<String> = keys.iter().map(|uid| format!("{}", uid)).collect();

        for (idx, uid) in keys.iter().enumerate() {
            write!(f, "{}: {}\n", idx, uid)?;
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::raft::tracker::inflights::Inflights;
    use crate::raft::tracker::progress::Progress;
    use crate::raft::tracker::state::StateType;

    #[test]
    fn it_process_string() {
        let mut ins = Inflights::new(1);
        ins.add(123);
        let mut pr = Progress {
            _match: 1,
            next: 2,
            state: StateType::StateSnapshot,
            pending_snapshot: 123,
            recent_active: false,
            probe_sent: true,
            is_leader: true,
            inflights: ins,
        };
        let exp = "StateSnapshot match=1 next=2 learner paused pendingSnap=123 inactive inflight=1[full]";
        assert_eq!(format!("{}", pr), exp);
    }

    #[test]
    pub fn t_process_is_paused() {
        struct Param {
            state: StateType,
            paused: bool,
            w: bool,
        }
        let tests = vec![Param {
            state: StateType::StateProbe,
            paused: false,
            w: false,
        }, Param {
            state: StateType::StateProbe,
            paused: true,
            w: true,
        }, Param {
            state: StateType::StateReplicate,
            paused: false,
            w: false,
        }, Param {
            state: StateType::StateSnapshot,
            paused: false,
            w: true,
        }, Param {
            state: StateType::StateReplicate,
            paused: true,
            w: false,
        }];

        // for (i, tt) in tests.iter().enumerate() {
        //     let p = Progress {
        //         _match: 0,
        //         next: 0,
        //         state: tt.state.clone(),
        //         pending_snapshot: 0,
        //         recent_active: false,
        //         probe_sent: tt.paused,
        //     };
        // }
    }
}