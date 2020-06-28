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


use crate::raft::raftpb::raft::{MessageType, HardState};

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