# fc — File Compression Tool

A gzip-compatible file compression/decompression CLI tool in Rust, built as [Codecrafters Rust Project 3](https://codecrafters.io/blog/rust-projects).

## Features

- **Compress** and **decompress** files with gzip-compatible output
- **Benchmark** compression performance with multi-iteration timing
- **Inspect** compressed file metadata
- Parallel frequency counting via `rayon`
- Property-based testing with `proptest`

## Installation

```bash
git clone https://github.com/durgesh-k-sharma/fc.git
cd fc
cargo build --release
```

The binary is at `target/release/fc`.

## Usage

### Compress a file

```bash
fc compress -i input.txt -o output.huff
```

With benchmark stats:

```bash
fc compress -i input.txt -o output.huff --benchmark
```

### Decompress a file

```bash
fc decompress -i output.huff -o restored.txt
```

### Benchmark

```bash
fc benchmark -i input.txt -n 10
```

### Show file info

```bash
fc info -i output.huff
```

## Gzip Compatibility

Compressed files use the standard gzip format (RFC 1952) and can be decompressed with `gunzip`:

```bash
gunzip -c output.huff > restored.txt
```

## Architecture

```
src/
├── core/          # Pure data structures (no I/O)
│   ├── bit_buffer.rs   # Bit-level read/write (LSB/MSB modes)
│   ├── frequency.rs    # Parallel byte frequency counting (rayon)
│   ├── huffman.rs      # Huffman tree: build, encode, decode
│   └── error.rs        # CompressionError enum (thiserror)
├── codec/         # CompressionAlgorithm trait + implementations
│   ├── trait.rs        # CompressionAlgorithm trait (extensible)
│   └── huffman_codec.rs# Huffman + DEFLATE + gzip framing
├── format/        # File format utilities
│   ├── header.rs       # Gzip header encode/decode (RFC 1952)
│   └── crc32.rs        # CRC32 checksum (crc32fast)
├── cli/           # Argument parsing + command dispatch
│   ├── args.rs         # clap derive structs
│   └── commands.rs     # compress, decompress, benchmark, info
├── io/            # Buffered I/O (placeholder for future)
├── bench/         # Benchmark harness (placeholder for future)
└── main.rs        # Binary entry point
```

The `CompressionAlgorithm` trait makes it straightforward to add new algorithms (e.g., LZW):

```rust
pub trait CompressionAlgorithm: Send + Sync {
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>, CompressionError>;
    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>, CompressionError>;
    fn algorithm_id(&self) -> AlgorithmId;
}
```

## Testing

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Property-based tests (proptest)
cargo test --test property

# Benchmarks
cargo bench
```

49 tests total: unit, integration, property-based, CLI, and gzip compatibility.

## Limitations

- DEFLATE stored blocks (type 00) are used — no LZ77 back-references or dynamic Huffman blocks. Compression ratio is modest compared to full deflate implementations.
- Inputs with all 256 unique byte values are rejected (format uses a `u8` symbol count field).
- The `verify` and `compare` flags on `decompress` and `benchmark` are accepted but not yet implemented.

## License

MIT
