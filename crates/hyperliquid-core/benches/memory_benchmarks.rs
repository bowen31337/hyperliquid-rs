//! Benchmarks for memory allocation optimizations
//!
//! This module provides benchmarks to measure the performance improvements
//! from memory allocation optimizations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

use hyperliquid_core::{
    memory::{ArenaAllocator, StringInterner, ZeroCopyValue, ObjectPool},
    types::{SymbolInterner, SymbolId, OptimizedOrder, OrderSide, OrderType, TradingAllocator},
};

/// Benchmark string interning vs regular string allocation
fn bench_string_interning(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_interning");

    // Benchmark regular string allocation
    group.bench_function("regular_strings", |b| {
        b.iter(|| {
            let symbols = vec!["BTC", "ETH", "SOL", "ADA", "XRP"];
            let mut strings = Vec::new();
            for _ in 0..1000 {
                for symbol in &symbols {
                    strings.push(symbol.to_string());
                }
            }
            black_box(strings);
        })
    });

    // Benchmark string interning
    group.bench_function("interned_strings", |b| {
        b.iter(|| {
            let mut interner = StringInterner::new();
            let symbols = vec!["BTC", "ETH", "SOL", "ADA", "XRP"];
            let mut ids = Vec::new();
            for _ in 0..1000 {
                for symbol in &symbols {
                    ids.push(interner.intern(symbol));
                }
            }
            black_box(ids);
        })
    });

    group.finish();
}

/// Benchmark arena allocation vs regular allocation
fn bench_arena_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena_allocation");

    // Benchmark regular allocation
    group.bench_function("regular_allocation", |b| {
        b.iter(|| {
            let mut orders = Vec::new();
            for i in 0..1000 {
                orders.push(OptimizedOrder {
                    symbol_id: SymbolId::from_raw(i % 5),
                    price: 100.0 + i as f64,
                    size: 1.0,
                    side: OrderSide::Buy,
                    order_type: OrderType::Limit,
                    metadata: None,
                });
            }
            black_box(orders);
        })
    });

    // Benchmark arena allocation
    group.bench_function("arena_allocation", |b| {
        b.iter(|| {
            let mut allocator = ArenaAllocator::new(64 * 1024);
            let mut ptrs = Vec::new();
            for i in 0..1000 {
                let order = OptimizedOrder {
                    symbol_id: SymbolId::from_raw(i % 5),
                    price: 100.0 + i as f64,
                    size: 1.0,
                    side: OrderSide::Buy,
                    order_type: OrderType::Limit,
                    metadata: None,
                };
                let ptr = allocator.allocate(order);
                ptrs.push(ptr);
            }
            black_box(ptrs);
        })
    });

    group.finish();
}

/// Benchmark object pooling vs regular allocation
fn bench_object_pooling(c: &mut Criterion) {
    let mut group = c.benchmark_group("object_pooling");

    // Benchmark regular allocation
    group.bench_function("regular_allocation", |b| {
        b.iter(|| {
            let mut orders = Vec::new();
            for _ in 0..1000 {
                orders.push(OptimizedOrder {
                    symbol_id: SymbolId::from_raw(0),
                    price: 100.0,
                    size: 1.0,
                    side: OrderSide::Buy,
                    order_type: OrderType::Limit,
                    metadata: None,
                });
            }
            // Simulate using and dropping
            black_box(orders);
        })
    });

    // Benchmark object pooling
    group.bench_function("object_pooling", |b| {
        b.iter(|| {
            let pool: ObjectPool<OptimizedOrder> = ObjectPool::new(100);
            let mut orders = Vec::new();
            for _ in 0..1000 {
                let mut order = pool.get();
                order.symbol_id = SymbolId::from_raw(0);
                order.price = 100.0;
                order.size = 1.0;
                order.side = OrderSide::Buy;
                order.order_type = OrderType::Limit;
                orders.push(order);
            }
            black_box(orders);
        })
    });

    group.finish();
}

/// Benchmark trading allocator vs separate allocators
fn bench_trading_allocator(c: &mut Criterion) {
    let mut group = c.benchmark_group("trading_allocator");

    // Benchmark separate allocators
    group.bench_function("separate_allocators", |b| {
        b.iter(|| {
            let mut symbol_interner = SymbolInterner::new();
            let mut arena = ArenaAllocator::new(64 * 1024);
            let mut pool: ObjectPool<OptimizedOrder> = ObjectPool::new(100);

            for i in 0..1000 {
                let symbol_id = symbol_interner.intern_symbol("BTC");
                let order = OptimizedOrder {
                    symbol_id,
                    price: 100.0 + i as f64,
                    size: 1.0,
                    side: OrderSide::Buy,
                    order_type: OrderType::Limit,
                    metadata: None,
                };
                let _ptr = arena.allocate(order);
                let _pooled_order = pool.get();
            }
        })
    });

    // Benchmark unified trading allocator
    group.bench_function("unified_trading_allocator", |b| {
        b.iter(|| {
            let mut allocator = TradingAllocator::new();
            for i in 0..1000 {
                let _order_ptr = allocator.allocate_order(
                    "BTC",
                    100.0 + i as f64,
                    1.0,
                    OrderSide::Buy,
                    OrderType::Limit,
                );
                let _pooled_order = allocator.pools.get_order();
            }
        })
    });

    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.measurement_time(Duration::from_secs(30));

    // Benchmark high-frequency symbol internment
    group.bench_function("high_freq_symbol_interning", |b| {
        b.iter(|| {
            let mut interner = StringInterner::new();
            let symbols = ["BTC", "ETH", "SOL", "ADA", "XRP", "DOGE", "DOT", "UNI", "LTC", "LINK"];

            for _ in 0..1000 {
                for symbol in &symbols {
                    let _id = interner.intern(symbol);
                }
            }

            black_box(interner.stats());
        })
    });

    // Benchmark arena reset performance
    group.bench_function("arena_reset", |b| {
        b.iter(|| {
            let mut allocator = ArenaAllocator::new(64 * 1024);

            // Allocate many objects
            for i in 0..1000 {
                let _ptr = allocator.allocate(i);
            }

            // Reset arena (simulating batch processing)
            allocator.reset();
        })
    });

    group.finish();
}

/// Benchmark zero-copy JSON parsing
fn bench_zero_copy_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("zero_copy_parsing");

    let json_data = r#"{"price": "100.50", "size": "1.0", "side": "buy", "symbol": "BTC"}"#;

    // Benchmark regular JSON parsing
    group.bench_function("regular_json_parsing", |b| {
        b.iter(|| {
            let parsed: serde_json::Value = black_box(serde_json::from_str(json_data).unwrap());
            let _price = parsed["price"].as_str().unwrap();
            let _size = parsed["size"].as_str().unwrap();
            let _side = parsed["side"].as_str().unwrap();
            let _symbol = parsed["symbol"].as_str().unwrap();
        })
    });

    // Benchmark zero-copy parsing
    group.bench_function("zero_copy_parsing", |b| {
        b.iter(|| {
            let parsed = ZeroCopyValue::from_json_str(black_box(json_data)).unwrap();
            if let ZeroCopyValue::Object(map) = parsed {
                let _price = map.get("price").unwrap().as_str().unwrap();
                let _size = map.get("size").unwrap().as_str().unwrap();
                let _side = map.get("side").unwrap().as_str().unwrap();
                let _symbol = map.get("symbol").unwrap().as_str().unwrap();
            }
        })
    });

    group.finish();
}

/// Benchmark different memory allocation strategies
fn bench_allocation_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_strategies");

    let strategies = [
        ("Vec allocation", |n: usize| {
            let mut vec = Vec::new();
            for i in 0..n {
                vec.push(i);
            }
            vec
        }),
        ("Arena allocation", |n: usize| {
            let mut arena = ArenaAllocator::new(64 * 1024);
            let mut ptrs = Vec::new();
            for i in 0..n {
                let ptr = arena.allocate(i);
                ptrs.push(ptr);
            }
            ptrs
        }),
    ];

    for (name, strategy) in strategies.iter() {
        group.bench_with_input(BenchmarkId::new(name, 1000), &1000, |b, &n| {
            b.iter(|| black_box(strategy(n)))
        });

        group.bench_with_input(BenchmarkId::new(name, 10000), &10000, |b, &n| {
            b.iter(|| black_box(strategy(n)))
        });

        group.bench_with_input(BenchmarkId::new(name, 100000), &100000, |b, &n| {
            b.iter(|| black_box(strategy(n)))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_string_interning,
    bench_arena_allocation,
    bench_object_pooling,
    bench_trading_allocator,
    bench_memory_usage,
    bench_zero_copy_parsing,
    bench_allocation_strategies
);

criterion_main!(benches);