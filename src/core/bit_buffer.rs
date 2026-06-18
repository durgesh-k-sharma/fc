#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitMode {
    LsbFirst,
    MsbFirst,
}

#[derive(Debug, Clone)]
pub struct BitBuffer {
    bytes: Vec<u8>,
    accumulator: u64,
    bit_count: u8, // bits currently in accumulator (0..=63)
    mode: BitMode,
}

impl BitBuffer {
    pub fn new(mode: BitMode) -> Self {
        Self {
            bytes: Vec::new(),
            accumulator: 0,
            bit_count: 0,
            mode,
        }
    }

    pub fn write_bits(&mut self, value: u64, num_bits: usize) {
        if num_bits == 0 {
            return;
        }
        debug_assert!(num_bits <= 64);

        for i in 0..num_bits {
            let bit = match self.mode {
                BitMode::LsbFirst => (value >> i) & 1,
                BitMode::MsbFirst => {
                    let width = if value == 0 { 0 } else { 64 - value.leading_zeros() };
                    let shift = width.saturating_sub(1 + i as u32);
                    (value >> shift) & 1
                }
            };
            let pos = match self.mode {
                BitMode::LsbFirst => self.bit_count,
                BitMode::MsbFirst => 7 - self.bit_count,
            };
            self.accumulator |= bit << pos;
            self.bit_count += 1;
            if self.bit_count == 8 {
                self.bytes.push(self.accumulator as u8);
                self.accumulator = 0;
                self.bit_count = 0;
            }
        }
    }

    pub fn flush(&mut self) {
        if self.bit_count > 0 {
            self.bytes.push(self.accumulator as u8);
            self.accumulator = 0;
            self.bit_count = 0;
        }
    }

    pub fn as_bytes(&mut self) -> Vec<u8> {
        self.flush();
        std::mem::take(&mut self.bytes)
    }

    pub fn bit_len(&self) -> usize {
        self.bytes.len() * 8 + self.bit_count as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_read_single_bit() {
        let mut buf = BitBuffer::new(BitMode::LsbFirst);
        buf.write_bits(1, 1);
        assert_eq!(buf.bit_len(), 1);
        let bytes = buf.as_bytes();
        assert_eq!(bytes, vec![0b0000_0001]);
    }

    #[test]
    fn write_read_multiple_bytes() {
        let mut buf = BitBuffer::new(BitMode::LsbFirst);
        buf.write_bits(0b10110011, 8);
        buf.write_bits(0b11110000, 8);
        let bytes = buf.as_bytes();
        assert_eq!(bytes, vec![0b10110011, 0b11110000]);
    }

    #[test]
    fn write_read_cross_byte_boundary() {
        let mut buf = BitBuffer::new(BitMode::LsbFirst);
        buf.write_bits(0xFF, 4);  // 4 bits set
        buf.write_bits(0x00, 4);  // 4 bits clear
        let bytes = buf.as_bytes();
        assert_eq!(bytes, vec![0b0000_1111]);
    }

    #[test]
    fn write_zero_bits_noop() {
        let mut buf = BitBuffer::new(BitMode::LsbFirst);
        buf.write_bits(0xFF, 0);
        assert_eq!(buf.bit_len(), 0);
        assert!(buf.as_bytes().is_empty());
    }

    #[test]
    fn flush_pads_remaining_bits() {
        let mut buf = BitBuffer::new(BitMode::LsbFirst);
        buf.write_bits(1, 1);  // 1 bit in accumulator
        buf.flush();
        assert_eq!(buf.as_bytes(), vec![0b0000_0001]);
    }

    #[test]
    fn msb_first_ordering() {
        let mut buf = BitBuffer::new(BitMode::MsbFirst);
        buf.write_bits(0b1100_0000, 4);  // write 1100
        let bytes = buf.as_bytes();
        assert_eq!(bytes, vec![0b1100_0000]);
    }
}
