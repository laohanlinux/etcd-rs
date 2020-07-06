// Copyright 2019 The etcd Authors
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

use crate::raft::tracker::{ProgressTracker, Config};
use crate::raft::raftpb::raft::ConfChangeSingle;
use crate::raft::tracker::progress::ProgressMap;
use crate::raft::quorum::joint::JointConfig;
use crate::raft::quorum::majority::MajorityConfig;
use bytes::BytesMut;
use std::fmt::Write;
use std::collections::HashMap;

// Changer facilitates configuration changes. It exposes methods to handle
// simple and joint consensus while performing the proper validation that allows
// refusing invalid configuration changes before they affect the active
// configuration.
pub struct Changer {
    pub tracker: ProgressTracker,
    pub last_index: u64,
}

impl Changer {
    pub fn enter_joint(&mut self, auto_leave: bool, ccs: Vec<ConfChangeSingle>) -> Result((Config, ProgressMap), String) {}
}

// pub(crate) fn symiff(l: HashMap<u64, ()>) -> u64 {
//     let mut n = 0;
//     let pairs = Vec::new(vec![MajorityConfig { votes: l.clone() }, MajorityConfig{votes: r.clone()}]);
// }

pub(crate) fn join(cfg: Config) -> bool {
    !outgoing(&cfg.voters).votes.is_empty()
}

pub(crate) fn incoming(voters: &JointConfig) -> &MajorityConfig {
    &voters.0[0]
}

pub(crate) fn outgoing(voters: &JointConfig) -> &MajorityConfig {
    &voters.0[1]
}

pub(crate) fn outgoing_ptr(ccs: Vec<ConfChangeSingle>) -> String {
    let mut buf = BytesMut::new();
    ccs.iter().map(|cs| {
        if !buf.is_empty() {
            buf.write_char(' ');
        }
        buf.write_str(&format!("{}({})", cs.get_field_type(), cs.get_node_id()));
    });
    String::from_utf8(buf.to_vec()).unwrap()
}

// prints the type and node_id of the configuration changes as a
// space-delimited string.
pub fn describe(ccs: Vec<ConfChangeSingle>) -> string {
    let mut buf = BytesMut::new();
    for cc in ccs {
        if !buf.is_empty() {
            buf.write_char(' ');
        }
        buf.write_str(&format!("{}({})", cc.get_field_type(), cc.get_node_id()));
    }
    String::from_utf8(buf.to_vec()).unwrap()
}