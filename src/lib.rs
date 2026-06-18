pub mod core;
pub mod codec;
pub mod format;
pub mod io;
pub mod cli;
pub mod bench;

pub use core::error::CompressionError;
pub use codec::r#trait::{CompressionAlgorithm, AlgorithmId};
