use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::codec::huffman_codec::HuffmanCodec;
use crate::codec::huffman_codec::decompress_bytes;
use crate::codec::r#trait::CompressionAlgorithm;
use crate::format::header::GzipHeader;

pub fn run_compress(
    input: &str,
    output: Option<&str>,
    benchmark: bool,
    verbose: bool,
    force: bool,
) -> Result<()> {
    let input_path = Path::new(input);
    if !input_path.exists() {
        anyhow::bail!("input file not found: {}", input);
    }

    let output = output.map(String::from).unwrap_or_else(|| {
        format!("{}.huff", input)
    });

    let output_path = Path::new(&output);
    if output_path.exists() && !force {
        anyhow::bail!("output file exists: {} (use --force to overwrite)", output);
    }

    let data = fs::read(input)
        .with_context(|| format!("failed to read input: {}", input))?;

    if verbose {
        eprintln!("Input: {} bytes", data.len());
    }

    let codec = HuffmanCodec;

    let start = Instant::now();
    let compressed = codec.compress(&data)
        .context("compression failed")?;
    let compress_time = start.elapsed();

    fs::write(&output, &compressed)
        .with_context(|| format!("failed to write output: {}", output))?;

    if verbose || benchmark {
        let ratio = (compressed.len() as f64 / data.len() as f64) * 100.0;
        let saved = 100.0 - ratio;
        let throughput = (data.len() as f64 / compress_time.as_secs_f64()) / 1_000_000.0;
        eprintln!("Input size:      {} bytes", data.len());
        eprintln!("Output size:     {} bytes", compressed.len());
        eprintln!("Ratio:           {:.1}% of original", ratio);
        eprintln!("Space saved:     {:.1}%", saved);
        eprintln!("Compress time:   {:.2} ms ({:.1} MB/s)",
            compress_time.as_secs_f64() * 1000.0, throughput);
    }

    Ok(())
}

pub fn run_decompress(
    input: &str,
    output: Option<&str>,
    _verify: bool,
    verbose: bool,
) -> Result<()> {
    let input_path = Path::new(input);
    if !input_path.exists() {
        anyhow::bail!("input file not found: {}", input);
    }

    let output = output.map(String::from).unwrap_or_else(|| {
        let s = input.strip_suffix(".huff").unwrap_or(input).to_string();
        if s == input {
            format!("{}.out", input)
        } else {
            s
        }
    });

    let data = fs::read(input)
        .with_context(|| format!("failed to read input: {}", input))?;

    if verbose {
        eprintln!("Input: {} bytes", data.len());
    }

    let start = Instant::now();
    let decompressed = decompress_bytes(&data)
        .context("decompression failed")?;
    let decompress_time = start.elapsed();

    fs::write(&output, &decompressed)
        .with_context(|| format!("failed to write output: {}", output))?;

    if verbose {
        let throughput = (decompressed.len() as f64 / decompress_time.as_secs_f64()) / 1_000_000.0;
        eprintln!("Output size:     {} bytes", decompressed.len());
        eprintln!("Decompress time: {:.2} ms ({:.1} MB/s)",
            decompress_time.as_secs_f64() * 1000.0, throughput);
    }

    Ok(())
}

pub fn run_benchmark(
    input: &str,
    iterations: usize,
    _compare: bool,
) -> Result<()> {
    let data = fs::read(input)
        .with_context(|| format!("failed to read input: {}", input))?;

    let codec = HuffmanCodec;

    let mut compress_times = Vec::new();
    let mut decompress_times = Vec::new();
    let mut compressed_size = 0;

    for i in 0..iterations {
        let start = Instant::now();
        let compressed = codec.compress(&data)?;
        let ct = start.elapsed();
        compress_times.push(ct);

        let start = Instant::now();
        let _decompressed = codec.decompress(&compressed)?;
        let dt = start.elapsed();
        decompress_times.push(dt);

        if i == 0 {
            compressed_size = compressed.len();
        }
    }

    let avg_compress: f64 = compress_times.iter().map(|d| d.as_secs_f64()).sum::<f64>() / iterations as f64;
    let avg_decompress: f64 = decompress_times.iter().map(|d| d.as_secs_f64()).sum::<f64>() / iterations as f64;
    let ratio = (compressed_size as f64 / data.len() as f64) * 100.0;

    println!("Benchmark Results ({} iterations):", iterations);
    println!("  Input size:        {} bytes", data.len());
    println!("  Output size:       {} bytes", compressed_size);
    println!("  Ratio:             {:.1}% of original", ratio);
    println!("  Avg compress:      {:.2} ms ({:.1} MB/s)",
        avg_compress * 1000.0,
        (data.len() as f64 / avg_compress) / 1_000_000.0);
    println!("  Avg decompress:    {:.2} ms ({:.1} MB/s)",
        avg_decompress * 1000.0,
        (data.len() as f64 / avg_decompress) / 1_000_000.0);

    Ok(())
}

pub fn run_info(input: &str) -> Result<()> {
    let data = fs::read(input)
        .with_context(|| format!("failed to read input: {}", input))?;

    if data.len() < 18 {
        anyhow::bail!("file too small to be a valid gzip file");
    }

    let header = GzipHeader::decode(&data[..10])?;
    let crc = u32::from_le_bytes([data[data.len()-8], data[data.len()-7], data[data.len()-6], data[data.len()-5]]);
    let size = u32::from_le_bytes([data[data.len()-4], data[data.len()-3], data[data.len()-2], data[data.len()-1]]);

    println!("File: {}", input);
    println!("  Compressed size:  {} bytes", data.len());
    println!("  Original size:    {} bytes", size);
    println!("  CRC32:            {:#010x}", crc);
    let os_str = match header.os {
        0 => "FAT".to_string(),
        3 => "Unix".to_string(),
        7 => "Macintosh".to_string(),
        11 => "NTFS".to_string(),
        255 => "Unknown".to_string(),
        other => format!("Other ({})", other),
    };
    println!("  OS:               {}", os_str);

    Ok(())
}
