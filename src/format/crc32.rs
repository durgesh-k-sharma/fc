pub fn crc32_compute(data: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vector() {
        // RFC 3309 test vector
        assert_eq!(crc32_compute(b"123456789"), 0xCBF43926);
    }

    #[test]
    fn empty_input() {
        assert_eq!(crc32_compute(b""), 0);
    }

    #[test]
    fn single_byte() {
        let result = crc32_compute(b"a");
        assert_ne!(result, 0);
    }
}
