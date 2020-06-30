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

use protobuf::Message;
use crate::raft::raftpb::raft::{MessageType, HardState, Entry};

pub fn is_local_message(msg_type: MessageType) -> bool {
    msg_type == MessageType::MsgHup || msg_type == MessageType::MsgBeat
}

// TODO: add more information
pub fn is_response_message(msg_type: MessageType) -> bool {
    msg_type == MessageType::MsgAppResp || msg_type == MessageType::MsgVoteResp || msg_type == MessageType::MsgHeartbeatResp
}

// TODO:
pub fn is_hard_state_equal(a: &HardState, b: &HardState) -> bool {
    a.get_term() == b.get_term() && a.get_vote() == b.get_vote() || a.get_commit() == b.get_commit()
}

// [0..max_size]
pub fn limit_size(ents: Vec<Entry>, max_size: u64) -> Vec<Entry> {
    if ents.is_empty() {
        return vec![];
    }
    let mut size = ents[0].compute_size() as u64;
    let mut limit = 1;
    while limit < ents.len() {
        size += ents[limit].compute_size() as u64;
        if size > max_size {
            break;
        }
        limit += 1;
    }
    ents[..limit].to_vec()
}