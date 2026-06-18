use crate::core::error::CompressionError;
use crate::core::frequency::FrequencyTable;
use crate::core::huffman::HuffmanTree;
use crate::codec::r#trait::{AlgorithmId, CompressionAlgorithm};
use crate::format::crc32::crc32_compute;
use crate::format::header::GzipHeader;

pub struct HuffmanCodec;

impl HuffmanCodec {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HuffmanCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl CompressionAlgorithm for HuffmanCodec {
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>, CompressionError> {
        compress_bytes(input)
    }

    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>, CompressionError> {
        decompress_bytes(input)
    }

    fn algorithm_id(&self) -> AlgorithmId {
        AlgorithmId::Huffman
    }
}

/// Compress input bytes using Huffman coding inside a gzip-compatible container.
///
/// Format:
/// - 10-byte gzip header
/// - DEFLATE stored block (type 00):
///   - 1 byte block header (BFINAL=1, BTYPE=00)
///   - 2 bytes LEN, 2 bytes NLEN
///   - Payload:
///     - 1 byte: number of symbols N
///     - N * 5 bytes: (symbol: u8, freq: u32) pairs
///     - 8 bytes: original bit length of Huffman-encoded data
///     - remaining: Huffman-encoded bitstream
/// - 8-byte gzip trailer (CRC32 + original size)
pub fn compress_bytes(input: &[u8]) -> Result<Vec<u8>, CompressionError> {
    if input.is_empty() {
        return Err(CompressionError::EmptyInput);
    }

    let crc = crc32_compute(input);
    let original_size = input.len() as u32;

    // Build frequency table
    let freqs = FrequencyTable::from_bytes_par(input);

    // Build Huffman tree
    let tree = HuffmanTree::from_frequencies(freqs.as_array())
        .ok_or(CompressionError::EmptyInput)?;

    // Encode data using the tree's encode method
    let encoded = tree.encode(input);
    // encoded format from tree.encode():
    //   single symbol: [symbol byte][8 bytes count]
    //   normal: [8 bytes bit_length][data bytes]

    // Build the stored block payload
    // Collect unique symbols and their frequencies
    let mut payload = Vec::new();

    // Count unique symbols
    let unique_symbols: Vec<(u8, u32)> = freqs
        .as_array()
        .iter()
        .enumerate()
        .filter(|(_, count)| **count > 0)
        .map(|(sym, &count)| (sym as u8, count as u32))
        .collect();

    let n = unique_symbols.len() as u8;
    payload.push(n);

    // Write each (symbol, freq) pair
    for (symbol, freq) in &unique_symbols {
        payload.push(*symbol);
        payload.extend_from_slice(&freq.to_le_bytes());
    }

    // Write the encoded data (which already has the 8-byte bit-length header from tree.encode)
    payload.extend_from_slice(&encoded);

    // Build DEFLATE stored block
    let mut deflate_data = Vec::new();

    // Block header: BFINAL=1, BTYPE=00 (stored)
    deflate_data.push(0b0000_0001);

    // LEN and NLEN
    let len = payload.len() as u16;
    let nlen = !len;
    deflate_data.extend_from_slice(&len.to_le_bytes());
    deflate_data.extend_from_slice(&nlen.to_le_bytes());
    deflate_data.extend_from_slice(&payload);

    // Build gzip file
    let mut output = Vec::new();

    // Header
    let header = GzipHeader { mtime: 0, os: 0xFF };
    output.extend_from_slice(&header.encode());

    // Compressed data
    output.extend_from_slice(&deflate_data);

    // Trailer
    output.extend_from_slice(&crc.to_le_bytes());
    output.extend_from_slice(&original_size.to_le_bytes());

    Ok(output)
}

pub fn decompress_bytes(input: &[u8]) -> Result<Vec<u8>, CompressionError> {
    if input.len() < 18 {
        return Err(CompressionError::Truncated {
            expected: 18,
            actual: input.len(),
        });
    }

    // Parse header
    let _header = GzipHeader::decode(&input[..10])?;

    // Extract trailer (last 8 bytes)
    let data_len = input.len();
    let expected_crc = u32::from_le_bytes([
        input[data_len - 8],
        input[data_len - 7],
        input[data_len - 6],
        input[data_len - 5],
    ]);
    let expected_size = u32::from_le_bytes([
        input[data_len - 4],
        input[data_len - 3],
        input[data_len - 2],
        input[data_len - 1],
    ]) as usize;

    // Extract DEFLATE data (between header and trailer)
    let deflate = &input[10..data_len - 8];

    // Parse stored block
    if deflate.is_empty() {
        return Err(CompressionError::InvalidData("empty DEFLATE data".into()));
    }

    let _bfinal = deflate[0] & 1;
    let btype = (deflate[0] >> 1) & 0b11;

    if btype != 0b00 {
        return Err(CompressionError::InvalidData(format!(
            "unsupported DEFLATE block type: {:02b}",
            btype
        )));
    }

    if deflate.len() < 5 {
        return Err(CompressionError::Truncated {
            expected: 5,
            actual: deflate.len(),
        });
    }

    let len = u16::from_le_bytes([deflate[1], deflate[2]]) as usize;
    let nlen = u16::from_le_bytes([deflate[3], deflate[4]]);

    if len != (!nlen as usize) {
        return Err(CompressionError::InvalidData(
            "stored block LEN/NLEN mismatch".into(),
        ));
    }

    if deflate.len() < 5 + len {
        return Err(CompressionError::Truncated {
            expected: 5 + len,
            actual: deflate.len(),
        });
    }

    let payload = &deflate[5..5 + len];

    // Parse the payload
    if payload.is_empty() {
        return Err(CompressionError::InvalidData("empty payload".into()));
    }

    let n = payload[0] as usize;
    let mut offset = 1;

    // Read frequency table
    let mut freqs = [0u64; 256];
    for _ in 0..n {
        if offset + 5 > payload.len() {
            return Err(CompressionError::Truncated {
                expected: offset + 5,
                actual: payload.len(),
            });
        }
        let symbol = payload[offset];
        let freq = u32::from_le_bytes([
            payload[offset + 1],
            payload[offset + 2],
            payload[offset + 3],
            payload[offset + 4],
        ]);
        freqs[symbol as usize] = freq as u64;
        offset += 5;
    }

    // The remaining bytes are the encoded data (with 8-byte bit-length header)
    let encoded_data = &payload[offset..];

    // Rebuild Huffman tree from frequencies
    let tree = HuffmanTree::from_frequencies(&freqs)
        .ok_or(CompressionError::InvalidData("failed to rebuild Huffman tree".into()))?;

    // Decode the data
    let decoded = tree.decode(encoded_data);

    // Verify CRC
    let actual_crc = crc32_compute(&decoded);
    if actual_crc != expected_crc {
        return Err(CompressionError::CrcMismatch {
            expected: expected_crc,
            actual: actual_crc,
        });
    }

    // Verify size
    if decoded.len() != expected_size {
        return Err(CompressionError::InvalidData(format!(
            "size mismatch: expected {}, got {}",
            expected_size,
            decoded.len()
        )));
    }

    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_decompress_roundtrip() {
        let input = b"hello world hello rust hello compression";
        let compressed = compress_bytes(input).unwrap();
        let decompressed = decompress_bytes(&compressed).unwrap();
        assert_eq!(decompressed, input.to_vec());
    }

    #[test]
    fn compress_produces_valid_gzip() {
        let input = b"test data for gzip format";
        let compressed = compress_bytes(input).unwrap();
        // Check gzip magic
        assert_eq!(compressed[0], 0x1f);
        assert_eq!(compressed[1], 0x8b);
        assert_eq!(compressed[2], 0x08); // DEFLATE
    }

    #[test]
    fn empty_input_errors() {
        let result = compress_bytes(b"");
        assert!(result.is_err());
    }

    #[test]
    fn single_byte_input() {
        let input = b"aaaaaaa";
        let compressed = compress_bytes(input).unwrap();
        let decompressed = decompress_bytes(&compressed).unwrap();
        assert_eq!(decompressed, input.to_vec());
    }

    #[test]
    fn large_text_roundtrip() {
        let input = "The quick brown fox jumps over the lazy dog. ".repeat(1000);
        let compressed = compress_bytes(input.as_bytes()).unwrap();
        let decompressed = decompress_bytes(&compressed).unwrap();
        assert_eq!(decompressed, input.as_bytes());
        // Should achieve some compression on repetitive text
        assert!(compressed.len() < input.len());
    }

    #[test]
    fn codec_trait_roundtrip() {
        let codec = HuffmanCodec::new();
        let input = b"trait-based compression test";
        let compressed = codec.compress(input).unwrap();
        let decompressed = codec.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input.to_vec());
    }
}
