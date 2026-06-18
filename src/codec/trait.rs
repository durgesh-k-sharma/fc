use crate::core::error::CompressionError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlgorithmId {
    Huffman,
}

pub trait CompressionAlgorithm: Send + Sync {
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>, CompressionError>;
    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>, CompressionError>;
    fn algorithm_id(&self) -> AlgorithmId;
}
