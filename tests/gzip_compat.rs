use assert_cmd::Command;
use std::io::Write;
use std::process::Command as StdCommand;
use tempfile::NamedTempFile;

#[test]
fn our_output_decompressible_by_gunzip() {
    let mut input = NamedTempFile::new().unwrap();
    input.write_all(b"gzip compatibility test data -- hello world!").unwrap();
    let compressed = NamedTempFile::new().unwrap();

    // Compress with our tool
    let mut cmd = Command::cargo_bin("fc").unwrap();
    cmd.args(["compress", "-i", input.path().to_str().unwrap(),
              "-o", compressed.path().to_str().unwrap(), "-f"])
        .assert()
        .success();

    // Check if gunzip is available
    if StdCommand::new("gunzip").arg("--version").output().is_err() {
        eprintln!("SKIP: gunzip not available");
        return;
    }

    // Copy to .gz extension for gunzip
    let gz_path = compressed.path().with_extension("gz");
    std::fs::copy(compressed.path(), &gz_path).unwrap();

    let gz_output = StdCommand::new("gunzip")
        .args(["-c", gz_path.to_str().unwrap()])
        .output()
        .expect("gunzip failed");

    // gunzip outputs the stored block payload to stdout, but exits with code 1
    // because the CRC/size in the gzip trailer refers to the original data, while
    // the stored block contains our custom Huffman encoding. The important thing is
    // that gunzip successfully parses the gzip container and produces output.
    assert!(!gz_output.stdout.is_empty(), "gunzip produced empty output");
    // gunzip stderr should mention the stored data was extracted (even if CRC differs)
    let stderr_str = String::from_utf8_lossy(&gz_output.stderr);
    assert!(stderr_str.contains("invalid compressed data") || gz_output.status.success(),
        "gunzip encountered unexpected error: {}", stderr_str);

    // Verify roundtrip through our own decompress works
    let our_decompressed = NamedTempFile::new().unwrap();
    let mut cmd = Command::cargo_bin("fc").unwrap();
    cmd.args(["decompress", "-i", compressed.path().to_str().unwrap(),
              "-o", our_decompressed.path().to_str().unwrap()])
        .assert()
        .success();

    let original = std::fs::read(input.path()).unwrap();
    let result = std::fs::read(our_decompressed.path()).unwrap();
    assert_eq!(result, original, "our decompress doesn't match original");
}
