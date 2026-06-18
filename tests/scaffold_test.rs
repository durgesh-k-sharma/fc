use fc::core::error::CompressionError;
use fc::codec::r#trait::{AlgorithmId, CompressionAlgorithm};

/// A minimal mock implementation to verify the trait is object-safe and usable.
struct MockAlgorithm;

impl CompressionAlgorithm for MockAlgorithm {
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>, CompressionError> {
        Ok(input.to_vec())
    }

    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>, CompressionError> {
        Ok(input.to_vec())
    }

    fn algorithm_id(&self) -> AlgorithmId {
        AlgorithmId::Huffman
    }
}

#[test]
fn error_display_messages() {
    let err = CompressionError::EmptyInput;
    assert_eq!(
        err.to_string(),
        "empty input: cannot compress zero-length data"
    );

    let err = CompressionError::InvalidData("bad header".into());
    assert_eq!(err.to_string(), "invalid compressed data: bad header");

    let err = CompressionError::CrcMismatch {
        expected: 0xDEAD,
        actual: 0xBEEF,
    };
    assert_eq!(
        err.to_string(),
        "corrupted data: CRC32 mismatch (expected 0xdead, got 0xbeef)"
    );

    let err = CompressionError::UnsupportedMethod(0xFF);
    assert_eq!(
        err.to_string(),
        "unsupported compression method: 0xff"
    );

    let err = CompressionError::Truncated {
        expected: 100,
        actual: 50,
    };
    assert_eq!(
        err.to_string(),
        "truncated input: expected 100 bytes, got 50"
    );
}

#[test]
fn algorithm_id_huffman() {
    let algo = MockAlgorithm;
    assert_eq!(algo.algorithm_id(), AlgorithmId::Huffman);
}

#[test]
fn algorithm_trait_compress_decompress() {
    let algo = MockAlgorithm;
    let data = b"hello world";
    let compressed = algo.compress(data).unwrap();
    let decompressed = algo.decompress(&compressed).unwrap();
    assert_eq!(decompressed, data);
}

#[test]
fn error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let comp_err: CompressionError = io_err.into();
    assert_eq!(comp_err.to_string(), "I/O error: file missing");
}
