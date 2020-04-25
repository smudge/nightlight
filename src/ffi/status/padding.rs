const BYTES: usize = 20;

/// Padding ... with some bytes
/// Helps avoid overflowing a C-struct
pub struct Padding {
    bytes: [u8; BYTES],
}

impl Default for Padding {
    fn default() -> Padding {
        Padding { bytes: [0; BYTES] }
    }
}

impl Padding {
    pub fn is_empty(&self) -> bool {
        self.bytes.iter().map(|i| *i as u64).sum::<u64>() != 0
    }
}
