use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fc::codec::huffman_codec::{compress_bytes, decompress_bytes};

fn bench_compress(c: &mut Criterion) {
    let sizes = [1024, 10_240, 102_400, 1_048_576];

    let mut group = c.benchmark_group("compress");
    for size in sizes {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        group.bench_function(format!("{}_bytes", size), |b| {
            b.iter(|| compress_bytes(black_box(&data)))
        });
    }
    group.finish();
}

fn bench_decompress(c: &mut Criterion) {
    let sizes = [1024, 10_240, 102_400];

    let mut group = c.benchmark_group("decompress");
    for size in sizes {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let compressed = compress_bytes(&data).unwrap();
        group.bench_function(format!("{}_bytes", size), |b| {
            b.iter(|| decompress_bytes(black_box(&compressed)))
        });
    }
    group.finish();
}

fn bench_roundtrip(c: &mut Criterion) {
    let text = "The quick brown fox jumps over the lazy dog. ".repeat(1000);
    let data = text.as_bytes();

    c.bench_function("roundtrip_text_44KB", |b| {
        b.iter(|| {
            let compressed = compress_bytes(black_box(data)).unwrap();
            let _decompressed = decompress_bytes(black_box(&compressed)).unwrap();
        })
    });
}

criterion_group!(benches, bench_compress, bench_decompress, bench_roundtrip);
criterion_main!(benches);
