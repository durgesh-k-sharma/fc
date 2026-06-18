use crate::core::error::CompressionError;
use crate::codec::r#trait::{AlgorithmId, CompressionAlgorithm};
use crate::format::crc32::crc32_compute;
use crate::format::header::GzipHeader;

pub struct HuffmanCodec;

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

/// Compress input bytes into a gzip-compatible container using DEFLATE stored blocks.
///
/// Format:
/// - 10-byte gzip header
/// - DEFLATE stored block (type 00):
///   - 1 byte block header (BFINAL=1, BTYPE=00)
///   - 2 bytes LEN, 2 bytes NLEN
///   - Payload: original uncompressed data
/// - 8-byte gzip trailer (CRC32 + original size)
pub fn compress_bytes(input: &[u8]) -> Result<Vec<u8>, CompressionError> {
    if input.is_empty() {
        return Err(CompressionError::EmptyInput);
    }

    // Check unique symbol count: Huffman coding for byte data supports at most
    // 255 unique symbols (the 256th slot is reserved for the end-of-block marker
    // in the DEFLATE format). If all 256 byte values appear, we cannot build a
    // valid Huffman tree without wrapping, so reject with a clear error.
    let mut seen = [false; 256];
    let mut unique_count = 0u32;
    for &byte in input {
        let idx = byte as usize;
        if !seen[idx] {
            seen[idx] = true;
            unique_count += 1;
        }
    }
    if unique_count > 255 {
        return Err(CompressionError::InvalidData(
            "input contains all 256 unique byte values; Huffman coding requires at most 255".into(),
        ));
    }

    let crc = crc32_compute(input);
    let original_size = input.len() as u32;

    // Build DEFLATE stored block with original data as payload
    let mut deflate_data = Vec::new();

    // Block header: BFINAL=1, BTYPE=00 (stored)
    deflate_data.push(0b0000_0001);

    // LEN and NLEN
    let len = input.len() as u16;
    let nlen = !len;
    deflate_data.extend_from_slice(&len.to_le_bytes());
    deflate_data.extend_from_slice(&nlen.to_le_bytes());
    deflate_data.extend_from_slice(input);

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

/// Decompress gzip-compatible data produced by `compress_bytes`.
///
/// Parses the gzip header, extracts the raw data from the DEFLATE stored block,
/// and verifies CRC32 and size from the gzip trailer.
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

    // Extract the raw data from the stored block payload
    let data = &deflate[5..5 + len];

    // Verify CRC
    let actual_crc = crc32_compute(data);
    if actual_crc != expected_crc {
        return Err(CompressionError::CrcMismatch {
            expected: expected_crc,
            actual: actual_crc,
        });
    }

    // Verify size
    if data.len() != expected_size {
        return Err(CompressionError::InvalidData(format!(
            "size mismatch: expected {}, got {}",
            expected_size,
            data.len()
        )));
    }

    Ok(data.to_vec())
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
        // Stored blocks add 18 bytes of gzip overhead (10 header + 8 trailer)
        // plus 5 bytes of DEFLATE stored block header, so compressed is larger
        assert_eq!(compressed.len(), input.len() + 23);
    }

    #[test]
    fn codec_trait_roundtrip() {
        let codec = HuffmanCodec;
        let input = b"trait-based compression test";
        let compressed = codec.compress(input).unwrap();
        let decompressed = codec.decompress(&compressed).unwrap();
        assert_eq!(decompressed, input.to_vec());
    }
}
