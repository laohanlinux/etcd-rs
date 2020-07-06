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

use crate::raft::raftpb::raft::{ConfState, ConfChangeSingle, ConfChange};
use crate::raft::raftpb::raft::ConfChangeType::{ConfChangeAddNode, ConfChangeRemoveNode, ConfChangeAddLearnerNode};

// toConfChangeSingle translates a conf state into 1) a slice of operations creating
// first the config that will become the outgoing one, and then the incoming one, and 
// b) another slice that, when applied to the config resulted from 1), respresents the 
// ConfState.
fn to_conf_change_single(cs: ConfState) -> (Vec<ConfChangeSingle>, Vec<ConfChangeSingle>){
    let mut out = Vec::new();
    let mut _in = Vec::new();
    for id in cs.get_voters_outgoing() {
        // If there are outgoing voters, first add them one by one so that the
        // (non-joint) config has them all.
        let mut out_cs_single = ConfChangeSingle::new();
        out_cs_single.set_node_id(*id);
        out_cs_single.set_field_type(ConfChangeAddNode);
        out.push(out_cs_single);
    }

    // We're done constructing the outgoing slice, now on to the incoming one
    // (which will apply on top of the config created by the outgoing slice).

    // First, we'll remove all of the outgoing voters.
    for id in cs.get_voters_outgoing() {
        let mut in_cs_single = ConfChangeSingle::new();
        in_cs_single.set_node_id(*id);
        in_cs_single.set_field_type(ConfChangeRemoveNode);
        _in.push(in_cs_single);
    }
    // Then we'll add the incoming voters and learners.
    for id in cs.get_voters() {
        let mut in_cs_single = ConfChangeSingle::new();
        in_cs_single.set_node_id(*id);
        in_cs_single.set_field_type(ConfChangeAddNode);
        _in.push(in_cs_single);
    }
    for id in cs.get_learners() {
        let mut in_cs_single = ConfChangeSingle::new();
        in_cs_single.set_node_id(*id);
        in_cs_single.set_field_type(ConfChangeRemoveNode);
        _in.push(in_cs_single);
    }
    // Same for LeanersNext; these are nodes we want to be learners but which
    // are currently voters in the outgoing config.
    for id in cs.get_learners_next() {
        let mut in_cs_single = ConfChangeSingle::new();
        in_cs_single.set_node_id(*id);
        in_cs_single.set_field_type(ConfChangeAddLearnerNode);
        _in.push(in_cs_single);
    }

    (out, _in)
}

// pub fn chain(chg ConfChange, ops: Vec<Fn(changer: Changer)>)