// revBytesLen is the byte length of a normal revision.
// First 8 bytes is the revision.main in big-endian format. The 9th byte 
// is a '_'. The last 8 bytes is the revision.sub in big-endian format.
const REV_BYTES_LEN: usize = 8 + 1 + 8;

// A revision indicates modification of the key-value space.
// The set of changes that share same main revision changes the key-value space atomically.
pub struct Revision {
    // main is the main revision of a set of changes that happen atomically.
    main: i64,
    // sub is the sub revision of a change in a set of changes that happen
    // atomically. Each change has different increasing sub revision in that
    // set.
    sub: i64,
}

impl Revision {
    pub fn greater_than(&self, b: &Revision) -> bool {
        if self.main > b.main {
            return true;
        }
        if self.main < b.main {
            return false;
        }
        return self.sub > b.sub;
    }
}
