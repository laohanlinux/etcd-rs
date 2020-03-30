use std::fmt::{self, Display, Formatter, Error};

// StateType is the state of a tracked follower.
pub enum StateType {
    // StateProbe indicates that a follower whose last index isn't known. Such a
    // follower is "probe" (i.e. an append sent periodically) to narrow down
    // its last index. In the ideal (and common) case, only one round of probing
    // is necessary as the follower will react with a hint. Followers that are
    // probed over extend periods of time are often offline.
    StateProbe,
    // StateReplicate is the steady in which a follower eagerly receives
    // log entries to append to its log.
    StateReplicate,
    // StateSnapshot indicates a follower that needs log entries not avaliable
    // from the leader's Raft log. Such a follower needs a full snapshot to
    // return a StateReplicate
    StateSnapshot,
}

impl Display for StateType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StateType::StateProbe => write!(f, "StateProbe"),
            StateType::StateReplicate => { write!(f, "StateReplicate") }
            StateType::StateSnapshot => write!(f, "StateSnapshot"),
        }
    }
}