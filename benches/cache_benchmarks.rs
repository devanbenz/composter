use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode, Throughput,
};
use rand::prelude::*;
use std::time::Duration;

use composter::clock_replacer::Replacer;
use composter::lru_replacer::LRUCache;

fn generate_sequential_access(size: usize, operations: usize) -> Vec<i32> {
    (0..operations as i32).collect()
}

fn generate_random_access(size: usize, operations: usize) -> Vec<i32> {
    let mut rng = StdRng::seed_from_u64(42); // Fixed seed for reproducibility
    (0..operations)
        .map(|_| rng.gen_range(0..size as i32))
        .collect()
}

fn generate_zipf_access(size: usize, operations: usize) -> Vec<i32> {
    let mut rng = StdRng::seed_from_u64(42);
    let mut accesses = Vec::with_capacity(operations);

    // Simplified Zipf distribution
    let ranks: Vec<f64> = (1..=size).map(|i| 1.0 / i as f64).collect();
    let sum: f64 = ranks.iter().sum();
    let probabilities: Vec<f64> = ranks.iter().map(|&r| r / sum).collect();

    for _ in 0..operations {
        let rand_val = rng.gen::<f64>();
        let mut cumsum = 0.0;
        let mut selected = 0;

        for (i, &prob) in probabilities.iter().enumerate() {
            cumsum += prob;
            if rand_val <= cumsum {
                selected = i;
                break;
            }
        }

        accesses.push(selected as i32);
    }

    accesses
}

fn bench_caches(c: &mut Criterion) {
    let mut group = c.benchmark_group("Cache Comparison");

    // Configure the benchmark group
    group.sampling_mode(SamplingMode::Flat);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(5));

    // Test different cache sizes
    let cache_sizes = vec![100, 1000, 10000];
    let operations = 100000;

    // Access patterns to test
    let patterns = vec![
        (
            "sequential",
            generate_sequential_access as fn(usize, usize) -> Vec<i32>,
        ),
        (
            "random",
            generate_random_access as fn(usize, usize) -> Vec<i32>,
        ),
        ("zipf", generate_zipf_access as fn(usize, usize) -> Vec<i32>),
    ];

    for &cache_size in &cache_sizes {
        group.throughput(Throughput::Elements(operations as u64));

        for (pattern_name, pattern_fn) in &patterns {
            let access_pattern = pattern_fn(cache_size, operations);

            // Benchmark Clock Replacement
            group.bench_with_input(
                BenchmarkId::new(format!("Clock_{}", pattern_name), cache_size),
                &access_pattern,
                |b, pattern| {
                    b.iter(|| {
                        let mut replacer = Replacer::new(cache_size);
                        for &value in pattern {
                            black_box(replacer.get(value));
                        }
                    })
                },
            );

            // Benchmark LRU Cache
            group.bench_with_input(
                BenchmarkId::new(format!("LRU_{}", pattern_name), cache_size),
                &access_pattern,
                |b, pattern| {
                    b.iter(|| {
                        let mut cache = LRUCache::new(cache_size);
                        for &value in pattern {
                            if black_box(cache.get(value)).is_none() {
                                cache.put(value, value);
                            }
                        }
                    })
                },
            );
        }
    }

    group.finish();
}

criterion_group!(benches, bench_caches);
criterion_main!(benches);
