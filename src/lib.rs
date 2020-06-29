#![feature(is_sorted)]

#[macro_use]
extern crate log;

use std::collections::HashMap;

pub mod lease;
pub mod mvcc;
pub mod raft;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
