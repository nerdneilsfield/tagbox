use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use tagbox_core::utils::{calculate_hash_from_bytes, HashType};

fn generate_test_data(size: usize) -> Vec<u8> {
    let mut rng = StdRng::seed_from_u64(42);
    let mut data = vec![0u8; size];
    rng.fill(&mut data[..]);
    data
}

fn benchmark_hash_algorithms(c: &mut Criterion) {
    let sizes = vec![
        ("1KB", 1024),
        ("1MB", 1024 * 1024),
        ("10MB", 10 * 1024 * 1024),
        ("100MB", 100 * 1024 * 1024),
    ];

    let algorithms = vec![
        ("MD5", HashType::Md5),
        ("SHA-256", HashType::Sha256),
        ("SHA-512", HashType::Sha512),
        ("Blake2b", HashType::Blake2b),
        ("Blake3", HashType::Blake3),
        ("XXHash3-64", HashType::XXH3_64),
        ("XXHash3-128", HashType::XXH3_128),
    ];

    for (size_name, size) in sizes {
        let data = generate_test_data(size);

        let mut group = c.benchmark_group(format!("hash_{}", size_name));

        for (algo_name, algo_type) in &algorithms {
            group.bench_with_input(
                BenchmarkId::from_parameter(algo_name),
                &(&data, *algo_type),
                |b, (data, hash_type)| {
                    b.iter(|| {
                        let _ = calculate_hash_from_bytes(black_box(data), black_box(*hash_type));
                    });
                },
            );
        }

        group.finish();
    }
}

fn benchmark_file_hash(c: &mut Criterion) {
    use std::fs;
    use std::io::Write;
    use tagbox_core::utils::calculate_file_hash_with_type;
    use tempfile::NamedTempFile;
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();

    // 创建测试文件
    let mut temp_file = NamedTempFile::new().unwrap();
    let test_data = generate_test_data(10 * 1024 * 1024); // 10MB
    temp_file.write_all(&test_data).unwrap();
    temp_file.flush().unwrap();

    let mut group = c.benchmark_group("file_hash_10MB");

    let algorithms = vec![
        ("Blake3", HashType::Blake3),
        ("Blake2b", HashType::Blake2b),
        ("SHA-256", HashType::Sha256),
        ("XXHash3-64", HashType::XXH3_64),
    ];

    for (algo_name, algo_type) in algorithms {
        group.bench_with_input(
            BenchmarkId::from_parameter(algo_name),
            &algo_type,
            |b, hash_type| {
                b.iter(|| {
                    rt.block_on(async {
                        let _ = calculate_file_hash_with_type(
                            black_box(temp_file.path()),
                            black_box(*hash_type),
                        )
                        .await;
                    });
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_hash_algorithms, benchmark_file_hash);
criterion_main!(benches);
