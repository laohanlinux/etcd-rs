#![feature(is_sorted)]
#![feature(const_fn)]

#[macro_use]
extern crate log;

use std::collections::HashMap;

pub mod lease;
pub mod mvcc;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
