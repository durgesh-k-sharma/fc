#[derive(thiserror::Error, Debug)]
pub enum CompressionError {
    #[error("empty input: cannot compress zero-length data")]
    EmptyInput,

    #[error("invalid compressed data: {0}")]
    InvalidData(String),

    #[error("corrupted data: CRC32 mismatch (expected {expected:#x}, got {actual:#x})")]
    CrcMismatch { expected: u32, actual: u32 },

    #[error("unsupported compression method: {0:#x}")]
    UnsupportedMethod(u8),

    #[error("truncated input: expected {expected} bytes, got {actual}")]
    Truncated { expected: usize, actual: usize },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
