use crate::raft::raftpb::raft::HardState;

type StateType = u64;

pub enum SnapshotStatus {
    Finish,
    Failure,
}

// const EMPTY_STATE: HardState = new_hard_state();
//
// const fn new_hard_state() -> HardState {
//     HardState::default()
// }

// SoftState provides state that is usefull for logging and debugging.
// The state is volatile and does not need to be persisted to the WAL.
#[derive(Eq)]
pub struct SoftState {
    pub lead: u64,
    // must be atomic operations to access; keep 64-bit aligned.
    pub raft_state: StateType,
}

impl PartialEq for SoftState {
    fn eq(&self, other: &Self) -> bool {
        self.lead == other.lead && self.raft_state == other.raft_state
    }
}