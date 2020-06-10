// Copyright 2015 The etcd Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


use crate::raft::tracker::state::StateType;
use crate::raft::raft::Raft;
use crate::raft::storage::Storage;
use crate::raft::raftpb::raft::HardState;

#[derive(Error, Debug, PartialEq)]
pub enum RawRaftError {
    // ErrStepLocalMsg is returned when try to step a local raft message
    #[error("raft: cannot step raft local message")]
    StepLocalMsg,
    // ErrStepPeerNotFound is returned when try to step a response message
    // but there is no peer found in raft.Prs for that node.
    #[error("raft: cannot step as peer not found")]
    StepPeerNotFound,
}

// SoftState provides state that is useful for logging and debugging.
// The state is volatile and does not need to be persisted to the WAL.
pub struct SoftState {
    // must use atomic operations to access; keep 64-bit aligned
    pub raft_state: StateType,
    pub lead: u64,
}

pub struct RawNode<S: Storage> {
    raft: Raft<S>,
    prev_soft_st: SoftState,
    prev_hard_st: HardState,
}

impl RawNode {
    // TODO
    // returns the current point-in-time state of this RawNode.
    pub fn read(&self) -> Ready {
        Ready::default()
    }

    // TODO
    // has_ready called when rawnode user need to check if any ready pending.
    // Your Code Here (2A).
    pub fn has_ready(&self) -> bool {
        false
    }

    // TODO
    // Advance notifies the RawNode that the application has applied and saved progress in the
    // last Ready result.
    pub fn advance(&self, rd: &Ready) -> Ready {
        // Your Code Here(2A).
        Ready::default()
    }

    // TODO
    // GetProgress return the the Progress of this node and its peers, if this
    // node is leader.

    // TODO
    // TransferLeader tris=es to transfer leadership to the given transferee.
    pub fn transfer_leader(&mut self, transferee: u64) {}
}


// Ready encapsulates
pub struct Ready {}

impl Default for Ready {
    fn default() -> Ready {
        Ready {}
    }
}