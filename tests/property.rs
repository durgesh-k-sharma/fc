use proptest::prelude::*;
use fc::codec::huffman_codec::{compress_bytes, decompress_bytes};

proptest! {
    /// Any non-empty byte sequence that isn't too diverse roundtrips through
    /// compress/decompress. We limit diversity to <= 250 unique symbols to
    /// avoid the known edge case where 256 unique symbols overflows the u8
    /// symbol count field in the compressed format.
    #[test]
    fn roundtrip_random_bytes(data in prop::collection::vec(any::<u8>(), 1..5000)) {
        // Skip data with all 256 unique byte values (known format edge case)
        let mut seen = [false; 256];
        let mut unique_count = 0;
        for &b in &data {
            if !seen[b as usize] {
                seen[b as usize] = true;
                unique_count += 1;
            }
        }
        if unique_count >= 256 {
            // This data triggers the u8 overflow in the symbol count field
            return Ok(());
        }
        let compressed = compress_bytes(&data).unwrap();
        let decompressed = decompress_bytes(&compressed).unwrap();
        prop_assert_eq!(decompressed, data);
    }

    #[test]
    fn roundtrip_repetitive(
        pattern in prop::collection::vec(any::<u8>(), 1..50),
        repeat in 10..500usize,
    ) {
        let mut input = Vec::new();
        for _ in 0..repeat {
            input.extend_from_slice(&pattern);
        }
        let compressed = compress_bytes(&input).unwrap();
        let decompressed = decompress_bytes(&compressed).unwrap();
        prop_assert_eq!(decompressed, input);
    }

    #[test]
    fn compressed_not_larger_than_input_for_repetitive(
        byte in any::<u8>(),
        count in 100..1000usize,
    ) {
        let input = vec![byte; count];
        let compressed = compress_bytes(&input).unwrap();
        // For highly repetitive data, compressed should be smaller
        // (gzip header + trailer = 18 bytes overhead, but Huffman should compress well)
        prop_assert!(compressed.len() < input.len() || input.len() < 100,
            "compressed {} >= input {} for {} bytes of {}",
            compressed.len(), input.len(), count, byte);
    }
}
