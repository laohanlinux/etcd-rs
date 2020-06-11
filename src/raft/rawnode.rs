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
use crate::raft::raft::{Raft, Progress, State};
use crate::raft::storage::Storage;
use crate::raft::raftpb::raft::{HardState, Message, MessageType, Entry, ConfChange, EntryType, ConfState};
use crate::raft::log::RaftLogError;
use crate::raft::util::{is_local_message, is_response_message};

use bytes::Bytes;
use protobuf::{RepeatedField, Message as Msg};
use thiserror::Error;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::error::Error as StdError;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum RawRaftError {
    // ErrStepLocalMsg is returned when try to step a local raft message
    #[error("raft: cannot step raft local message")]
    StepLocalMsg,
    // ErrStepPeerNotFound is returned when try to step a response message
    // but there is no peer found in raft.Prs for that node.
    #[error("raft: cannot step as peer not found")]
    StepPeerNotFound,
    #[error("unknown error: {0}")]
    StepUnknown(String),
}

// SoftState provides state that is useful for logging and debugging.
// The state is volatile and does not need to be persisted to the WAL.
pub struct SoftState {
    // must use atomic operations to access; keep 64-bit aligned
    pub raft_state: StateType,
    pub lead: u64,
}

// pub fn new_raw_node<S: Storage>(config: Config) -> Result<RawNode<S>, String> {
//     Ok(RawNode::default())
// }

// RawNode is a thread-unsafe Node.
// The methods of this struct corresponds to the methods of Node and are described
// more fully there.
pub struct RawNode<S: Storage> {
    raft: Raft<S>,
    // prev_soft_st: SoftState,
    // prev_hard_st: HardState,
}

// NewRawNode instance a RawNode from the given configuration.
//
// See Bootstrap() for bootstrapping an initial state; this replaces the follower
// `peers` argument to this method (with identical behavior). However, It is
// recommended that instead of calling Bootstrap. application bootstrap their
// state manually by setting up a Storage that has a first index > 1 and which
// stores the described ConfState as its InitialState.
impl<S: Storage> RawNode<S> {
    // Tick advances the interval logical clock by a single tick.
    pub fn tick(&mut self) {
        self.raft.tick();
    }

    // Campaign causes this RawNode to transition to candidate state.
    pub fn campaign(&mut self) -> Result<()> {
        let mut msg = Message::new();
        msg.set_field_type(MessageType::MsgHup);
        self.raft.step(msg).map_err(|err| anyhow!("{}", err))
    }

    // Propose propose data be appended to the raft log.
    pub fn propose(&mut self, data: Bytes) -> Result<()> {
        let mut ent = Entry::new();
        ent.set_Data(data);
        let mut msg = Message::new();
        msg.set_field_type(MessageType::MsgProp);
        msg.set_from(self.raft.id);
        msg.set_entries(RepeatedField::from_slice(&vec![ent]));
        self.raft.step(msg).map_err(|err| anyhow!("{}", err))
    }

    // ProposeConfChange proposes a config message.
    pub fn propose_conf_change(&mut self, conf_change: &ConfChange) -> Result<()> {
        let data = Bytes::from(conf_change.write_to_bytes().unwrap());
        let mut entry = Entry::new();
        entry.set_Type(EntryType::EntryConfChange); // TODO: use x11EntryConfChangeV2
        entry.set_Data(data);
        let mut msg = Message::new();
        msg.set_field_type(MessageType::MsgProp);
        msg.set_entries(RepeatedField::from_slice(&vec![entry]));
        self.raft.step(msg).map_err(|err| anyhow!("{}", err))
    }

    // ApplyChange applies a config change to the local node.
    // pub fn apply_conf_change(&mut self, conf_change: ConfChange) -> ConfState {
    //     if !conf_change.has_node_id() {
    //         let mut conf_state = ConfState::new();
    //         conf_state.set_voters()
    //     }
    // }

    // Step advances the state machine using the given message.
    pub fn step(&mut self, m: Message) -> ::std::result::Result<(), RawRaftError> {
        // ignore unexpected local message receiving over the network
        if is_local_message(m.get_field_type()) {
            return Err(RawRaftError::StepLocalMsg);
        }
        if self.raft.prs.contains_key(&m.get_from()) || !is_response_message(m.get_field_type()) {
            return self.raft.step(m).map_err(|err| RawRaftError::StepUnknown(err));
        }
        Err(RawRaftError::StepPeerNotFound)
    }

    // TODO
    // returns the current point-in-time state of this RawNode.
    pub fn ready(&self) -> Ready {
        // Your Code Here (2A).
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
    pub fn get_progress(&self) -> HashMap<u64, Progress> {
        let mut prs = HashMap::new();
        if self.raft.state == State::Leader {
            prs = self.raft.prs.clone();
        }
        prs
    }

    // TODO
    // TransferLeader tris=es to transfer leadership to the given transferee.
    pub fn transfer_leader(&mut self, transferee: u64) {
        let mut msg = Message::new();
        msg.set_field_type(MessageType::MsgTransferLeader);
        msg.set_from(transferee);
        self.raft.step(msg);
    }
}


// Ready encapsulates
pub struct Ready {}

impl Default for Ready {
    fn default() -> Ready {
        Ready {}
    }
}