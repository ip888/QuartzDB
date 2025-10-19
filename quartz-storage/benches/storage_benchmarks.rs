use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use quartz_storage::{StorageConfig, StorageEngine};
use std::sync::Arc;
use tempfile::TempDir;

/// Create a test storage engine for benchmarking
fn create_bench_storage(config: StorageConfig) -> (Arc<StorageEngine>, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let path = temp_dir.path().to_str().unwrap();
    let engine = Arc::new(
        StorageEngine::with_config(path, config).expect("Failed to create storage engine"),
    );
    (engine, temp_dir)
}

/// Benchmark: Single write operations with various cache sizes
fn bench_write_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_operations");

    for cache_size in [100, 1000, 10000] {
        let config = StorageConfig {
            cache_size,
            enable_wal: true,
            ..Default::default()
        };

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("put_with_wal", cache_size),
            &cache_size,
            |b, _| {
                let (engine, _temp) = create_bench_storage(config.clone());
                let rt = tokio::runtime::Runtime::new().unwrap();
                let mut counter = 0u64;

                b.iter(|| {
                    rt.block_on(async {
                        let key = format!("bench_key_{}", counter);
                        let value = format!("bench_value_{}", counter);
                        counter += 1;
                        engine
                            .put(black_box(key.as_bytes()), black_box(value.as_bytes()))
                            .await
                            .expect("Put failed");
                    });
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Batch write operations
fn bench_batch_writes(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_writes");

    for batch_size in [10, 100, 1000] {
        group.throughput(Throughput::Elements(batch_size));
        group.bench_with_input(
            BenchmarkId::new("batch_put", batch_size),
            &batch_size,
            |b, &size| {
                let config = StorageConfig {
                    cache_size: 10000,
                    enable_wal: true,
                    ..Default::default()
                };
                let (engine, _temp) = create_bench_storage(config);
                let rt = tokio::runtime::Runtime::new().unwrap();

                b.iter(|| {
                    rt.block_on(async {
                        for i in 0..size {
                            let key = format!("batch_key_{}", i);
                            let value = format!("batch_value_{}", i);
                            engine
                                .put(key.as_bytes(), value.as_bytes())
                                .await
                                .expect("Put failed");
                        }
                    });
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Read operations (cache hits vs misses)
fn bench_read_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_operations");

    // Benchmark cache hits
    {
        let config = StorageConfig {
            cache_size: 10000,
            enable_wal: false,
            ..Default::default()
        };
        let (engine, _temp) = create_bench_storage(config);
        let rt = tokio::runtime::Runtime::new().unwrap();

        // Pre-populate data
        rt.block_on(async {
            for i in 0..1000 {
                let key = format!("read_key_{}", i);
                let value = format!("read_value_{}", i);
                engine
                    .put(key.as_bytes(), value.as_bytes())
                    .await
                    .expect("Put failed");
            }
        });

        group.throughput(Throughput::Elements(1));
        group.bench_function("get_cache_hit", |b| {
            let mut counter = 0u64;
            b.iter(|| {
                rt.block_on(async {
                    let key = format!("read_key_{}", counter % 1000);
                    counter += 1;
                    let _ = engine.get(black_box(key.as_bytes())).await;
                });
            });
        });
    }

    // Benchmark cache misses
    {
        let config = StorageConfig {
            cache_size: 10,
            enable_wal: false,
            ..Default::default()
        };
        let (engine, _temp) = create_bench_storage(config);
        let rt = tokio::runtime::Runtime::new().unwrap();

        // Pre-populate data
        rt.block_on(async {
            for i in 0..1000 {
                let key = format!("miss_key_{}", i);
                let value = format!("miss_value_{}", i);
                engine
                    .put(key.as_bytes(), value.as_bytes())
                    .await
                    .expect("Put failed");
            }
        });

        group.throughput(Throughput::Elements(1));
        group.bench_function("get_cache_miss", |b| {
            let mut counter = 0u64;
            b.iter(|| {
                rt.block_on(async {
                    let key = format!("miss_key_{}", counter % 1000);
                    counter += 1;
                    let _ = engine.get(black_box(key.as_bytes())).await;
                });
            });
        });
    }

    group.finish();
}

/// Benchmark: Delete operations
fn bench_delete_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("delete_operations");

    let config = StorageConfig {
        cache_size: 1000,
        enable_wal: true,
        ..Default::default()
    };
    let (engine, _temp) = create_bench_storage(config);
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Pre-populate data
    rt.block_on(async {
        for i in 0..10000 {
            let key = format!("del_key_{}", i);
            let value = format!("del_value_{}", i);
            engine
                .put(key.as_bytes(), value.as_bytes())
                .await
                .expect("Put failed");
        }
    });

    group.throughput(Throughput::Elements(1));
    group.bench_function("delete", |b| {
        let mut counter = 0u64;
        b.iter(|| {
            rt.block_on(async {
                let key = format!("del_key_{}", counter);
                counter += 1;
                engine
                    .delete(black_box(key.as_bytes()))
                    .await
                    .expect("Delete failed");
            });
        });
    });

    group.finish();
}

/// Benchmark: WAL enabled vs disabled
fn bench_wal_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("wal_comparison");

    for enable_wal in [false, true] {
        let config = StorageConfig {
            cache_size: 1000,
            enable_wal,
            ..Default::default()
        };

        let label = if enable_wal {
            "wal_enabled"
        } else {
            "wal_disabled"
        };

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::new("write", label), &enable_wal, |b, _| {
            let (engine, _temp) = create_bench_storage(config.clone());
            let rt = tokio::runtime::Runtime::new().unwrap();
            let mut counter = 0u64;

            b.iter(|| {
                rt.block_on(async {
                    let key = format!("wal_key_{}", counter);
                    let value = format!("wal_value_{}", counter);
                    counter += 1;
                    engine
                        .put(black_box(key.as_bytes()), black_box(value.as_bytes()))
                        .await
                        .expect("Put failed");
                });
            });
        });
    }

    group.finish();
}

/// Benchmark: Concurrent operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");

    for num_tasks in [4, 8, 16] {
        group.throughput(Throughput::Elements(num_tasks));
        group.bench_with_input(
            BenchmarkId::new("concurrent_writes", num_tasks),
            &num_tasks,
            |b, &tasks| {
                let config = StorageConfig {
                    cache_size: 10000,
                    enable_wal: false,
                    ..Default::default()
                };
                let (engine, _temp) = create_bench_storage(config);
                let rt = tokio::runtime::Runtime::new().unwrap();

                b.iter(|| {
                    rt.block_on(async {
                        let mut handles = vec![];

                        for task_id in 0..tasks {
                            let engine_clone = Arc::clone(&engine);
                            let handle = tokio::spawn(async move {
                                let key = format!("concurrent_key_{}", task_id);
                                let value = format!("concurrent_value_{}", task_id);
                                engine_clone
                                    .put(key.as_bytes(), value.as_bytes())
                                    .await
                                    .expect("Put failed");
                            });
                            handles.push(handle);
                        }

                        for handle in handles {
                            handle.await.expect("Task failed");
                        }
                    });
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Mixed workload (read/write ratio)
fn bench_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_workload");

    let config = StorageConfig {
        cache_size: 5000,
        enable_wal: false,
        ..Default::default()
    };
    let (engine, _temp) = create_bench_storage(config);
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Pre-populate data
    rt.block_on(async {
        for i in 0..1000 {
            let key = format!("mixed_key_{}", i);
            let value = format!("mixed_value_{}", i);
            engine
                .put(key.as_bytes(), value.as_bytes())
                .await
                .expect("Put failed");
        }
    });

    group.throughput(Throughput::Elements(10));
    group.bench_function("read_write_70_30", |b| {
        let mut counter = 0u64;
        b.iter(|| {
            rt.block_on(async {
                // 70% reads, 30% writes
                for i in 0..10 {
                    let key = format!("mixed_key_{}", (counter + i) % 1000);

                    if i < 7 {
                        // Read
                        let _ = engine.get(key.as_bytes()).await;
                    } else {
                        // Write
                        let value = format!("updated_value_{}", counter + i);
                        engine
                            .put(key.as_bytes(), value.as_bytes())
                            .await
                            .expect("Put failed");
                    }
                }
                counter += 10;
            });
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_write_operations,
    bench_batch_writes,
    bench_read_operations,
    bench_delete_operations,
    bench_wal_comparison,
    bench_concurrent_operations,
    bench_mixed_workload
);
criterion_main!(benches);
