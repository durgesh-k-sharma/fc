use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn cli_compress_creates_output() {
    let mut input = NamedTempFile::new().unwrap();
    input.write_all(b"cli test data for compression").unwrap();
    let output = NamedTempFile::new().unwrap();

    let mut cmd = Command::cargo_bin("fc").unwrap();
    cmd.args(["compress", "-i", input.path().to_str().unwrap(),
              "-o", output.path().to_str().unwrap(), "-f"])
        .assert()
        .success();

    // Output file should exist and be non-empty
    let metadata = std::fs::metadata(output.path()).unwrap();
    assert!(metadata.len() > 0);
}

#[test]
fn cli_decompress_roundtrip() {
    let mut input = NamedTempFile::new().unwrap();
    input.write_all(b"cli roundtrip test data").unwrap();
    let compressed = NamedTempFile::new().unwrap();
    let decompressed = NamedTempFile::new().unwrap();

    // Compress
    let mut cmd = Command::cargo_bin("fc").unwrap();
    cmd.args(["compress", "-i", input.path().to_str().unwrap(),
              "-o", compressed.path().to_str().unwrap(), "-f"])
        .assert()
        .success();

    // Decompress
    let mut cmd = Command::cargo_bin("fc").unwrap();
    cmd.args(["decompress", "-i", compressed.path().to_str().unwrap(),
              "-o", decompressed.path().to_str().unwrap()])
        .assert()
        .success();

    // Verify content
    let original = std::fs::read_to_string(input.path()).unwrap();
    let result = std::fs::read_to_string(decompressed.path()).unwrap();
    assert_eq!(original, result);
}

#[test]
fn cli_info_shows_metadata() {
    let mut input = NamedTempFile::new().unwrap();
    input.write_all(b"info test data").unwrap();
    let compressed = NamedTempFile::new().unwrap();

    // Compress first
    let mut cmd = Command::cargo_bin("fc").unwrap();
    cmd.args(["compress", "-i", input.path().to_str().unwrap(),
              "-o", compressed.path().to_str().unwrap(), "-f"])
        .assert()
        .success();

    // Info
    let mut cmd = Command::cargo_bin("fc").unwrap();
    cmd.args(["info", "-i", compressed.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Compressed size"));
}

#[test]
fn cli_missing_input_errors() {
    let mut cmd = Command::cargo_bin("fc").unwrap();
    cmd.args(["compress", "-i", "nonexistent.txt"])
        .assert()
        .failure();
}
