const BYTES: usize = 32;

use std::os::raw::c_uchar;

/// Padding ... with some bytes:
/// Helps us know when the OS writes
/// a bigger struct than we expect
pub struct Padding {
    bytes: [c_uchar; BYTES],
}

impl Default for Padding {
    fn default() -> Padding {
        Padding { bytes: [0; BYTES] }
    }
}

impl Padding {
    pub fn is_empty(&self) -> bool {
        eprintln!("PADDING: {:?}", self.bytes);
        self.bytes.iter().map(|i| *i as u64).sum::<u64>() == 0
    }
}
