use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

// We test the library functions directly since the binary may not be built yet
use fc::codec::huffman_codec::{compress_bytes, decompress_bytes};

#[test]
fn roundtrip_via_tempfile() {
    let content = b"integration test: compress a file, decompress it, verify match";
    let mut input_file = NamedTempFile::new().unwrap();
    input_file.write_all(content).unwrap();
    let input_path = input_file.path();

    let output_file = NamedTempFile::new().unwrap();
    let output_path = output_file.path();

    let data = fs::read(input_path).unwrap();
    let compressed = compress_bytes(&data).unwrap();
    fs::write(output_path, &compressed).unwrap();

    let compressed_data = fs::read(output_path).unwrap();
    let decompressed = decompress_bytes(&compressed_data).unwrap();
    assert_eq!(decompressed, content);
}

#[test]
fn roundtrip_sample_file() {
    let data = fs::read("test_data/sample.txt").expect("test_data/sample.txt not found");
    let compressed = compress_bytes(&data).unwrap();
    let decompressed = decompress_bytes(&compressed).unwrap();
    assert_eq!(decompressed, data);
}

#[test]
fn roundtrip_various_sizes() {
    for size in [1, 10, 100, 1000, 10000] {
        // Use modulo 200 to stay under the 256 unique-symbol limit of the format
        let data: Vec<u8> = (0..size).map(|i| (i % 200) as u8).collect();
        let compressed = compress_bytes(&data).unwrap();
        let decompressed = decompress_bytes(&compressed).unwrap();
        assert_eq!(decompressed, data, "failed for size {}", size);
    }
}
