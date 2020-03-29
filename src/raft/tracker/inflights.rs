#[derive(Default, Clone)]
pub struct Inflights {
    // the starting index in the buffer
    start: usize,
    // number of inflights in the buffer
    count: isize,
    // the size of the buffer
    size: isize,
    // buffer contains the index of the last entry
    // inside one message
    buffer: Vec<u64>,
}

impl Inflights {
    pub fn new(size: isize) -> Self {
        Inflights { start: 0, count: 0, size, buffer: vec![] }
    }

    pub fn full(&mut self) -> bool {
        self.count == self.size
    }

    pub fn free_le(&mut self, to: u64) {
        if self.count == 0 || to < self.buffer[self.start] {
            // out of the left side of the window
            return;
        }

        let mut idx = self.start;
        let mut i = 0;
        for i in 0..self.count {

        }
    }

    // FreeFirstOne releases the first inflight. This is a no-op if nothing is
    // inflight.
    pub fn free_first_one(&mut self) {}

    pub fn count(&self) -> isize {
        self.count
    }

    fn reset(&mut self) {
        self.count = 0;
        self.start = 0;
    }
}