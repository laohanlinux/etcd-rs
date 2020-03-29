// Inflights limits the number of MsgApp(represented by the largest index
// contained within) sent to followers but not yet acknowledged by them. Callers
// use Full() to check whether more messages can be sent, call Add() whenever
// the are sending a new append, and release "quota" via free_le() whenever an
// ack is received.
#[derive(Default, PartialEq, Clone, Debug)]
pub struct Inflights {
    // the starting index in the buffer
    start: usize,
    // number of inflights in the buffer
    count: usize,
    // the size of the buffer
    size: usize,
    // buffer contains the index of the last entry
    // inside one message
    buffer: Vec<u64>,
}

impl Inflights {
    pub fn new(size: usize) -> Self {
        Inflights { start: 0, count: 0, size, buffer: vec![] }
    }

    // Add notifies the Inflights that a new message with the given index is being
    // dispatched. Full() must be called prior to Add() to verify that there is room
    // for one more message, and consecutive calls to add Add() must provide a
    // monotonic sequence of indexes.
    pub fn add(&mut self, inflight: u64) {
        if self.full() {
            panic!("cannot add into a Full inflights");
        }
        let mut next = self.start + self.count;
        let mut size = self.size;
        if next >= size {
            next -= size;
        }
        if next >= self.buffer.len() {
            self.grow();
        }
        self.buffer[next] = inflight;
        self.count += 1;
    }

    // grow the inflight buffer by doubling up tp inflights.size. We grow on demand
    // instead of preallocating to inflights.size to handle system which have
    // thousands of Raft groups per process.
    pub fn grow(&mut self) {
        let mut new_size = self.buffer.len() * 2;
        if new_size == 0 {
            new_size = 1;
        } else if new_size > self.size {
            new_size = self.size;
        }
        let mut new_buffer = Vec::with_capacity(new_size);
        new_buffer.extend_from_slice(&self.buffer);
        self.buffer = new_buffer;
    }

    // FreeLe frees the inflights smaller or equal to the given `to` flight.
    pub fn free_le(&mut self, to: u64) {
        if self.count == 0 || to < self.buffer[self.start] {
            // out of the left side of the window
            return;
        }

        let mut idx = self.start;
        let mut i = 0;
        while i < self.count {
            if to < self.buffer[idx] { // found the first large inflight
                break;
            }
            let size = self.size;
            idx + 1;
            if idx >= size {
                idx -= size;
            }
            i += 1;
        }
        // free i inflights and set new start index
        self.count -= i;
        self.start = idx;
        if self.count == 0 {
            // inflights is empty, reset the start index so that we don't grow the
            // buffer unnecessarily.
            self.start = 0;
        }
    }

    // FreeFirstOne releases the first inflight. This is a no-op if nothing is
    // inflight.
    pub fn free_first_one(&mut self) {}

    pub fn full(&mut self) -> bool {
        self.count == self.size
    }

    pub fn count(&self) -> usize {
        self.count
    }

    fn reset(&mut self) {
        self.count = 0;
        self.start = 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::raft::tracker::inflights::Inflights;

    #[test]
    fn it_inflights_add() {
        let mut inf = Inflights {
            start: 0,
            count: 0,
            size: 10,
            buffer: vec![0].repeat(10),
        };
        for i in 0..5 {
            inf.add(i);
        }
        let mut want_inf = Inflights {
            start: 0,
            count: 5,
            size: 10,
            buffer: vec![0, 1, 2, 3, 4, 0, 0, 0, 0, 0],
        };
        assert_eq!(inf, want_inf);
    }
}
