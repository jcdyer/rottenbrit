
pub struct BitField{
    inner: Vec<u8>,
    size: usize,
}

impl BitField {
    pub fn new(size) -> BitField {
        BitField {
            inner: vec![0; (size + 7) / 8],
            size
        }
    }

    pub get_first_unset_from(idx: usize) -> usize {
        let start = idx / 8;
        let offset = idx % 8;
        let byte = self.inner[start..];
        if self.inner[start] != 0xff {
            for x in 0..(8 - offset) {
                if self.get(idx + x) {
                    return Some(idx + x);
                }
            }
        }
        for byte in start..self.inner.len() {
            if let Some(x) = self.get_unset_in_byte(byte) {
                return Some(8 * byte + x)
            }
        }
        // Special handling for final partial byte
        for byte in 0..start {
            self.get_unset_in_byte(byte)
            if let Some(x) = self.get_unset_in_byte(byte) {
                return Some(8 * byte + x)
            }
        }
    }

    pub get(&self, idx: usize) -> bool {
        let byte = idx / 8;
        self.inner[byte] & self.get_mask(idx) > 0
    }

    pub set(&mut self, idx: usize, value: bool) {
        let byte = idx / 8;
        self.inner[byte] |= self.get_mask(idx)
    }

    fn get_unset_in_byte(&self, byte: usize) -> Option<usize> {
        if byte == 0xff {
            None
        } else {
            for offset in 0..8 {
                if byte & (1 << offset) {
                    return Some(offset);
                }
            }
            unreachable!()
        }
    }

    fn get_mask(&self, idx: usize) -> u8 {
        1 << (idx % 8) 
    }
}
