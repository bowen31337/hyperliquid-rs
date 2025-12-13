//! Integration tests for memory allocation optimizations
//!
//! These tests verify that memory allocation optimizations work correctly
//! and provide the expected performance benefits.

use std::sync::Arc;
use std::thread;
use std::time::Duration;
use hyperliquid_core::{
    memory::{ArenaAllocator, StringInterner, ZeroCopyValue, ObjectPool, MemoryProfiler},
    types::{SymbolInterner, SymbolId, OptimizedOrder, OrderSide, OrderType, TradingAllocator},
};

#[test]
fn test_memory_profiling() {
    let profiler = MemoryProfiler::new(Duration::from_millis(100));

    // Start profiling
    profiler.start();

    // Simulate memory-intensive operations
    thread::sleep(Duration::from_millis(250));

    // Allocate some memory
    let mut allocator = ArenaAllocator::new(64 * 1024);
    for i in 0..1000 {
        let _ptr = allocator.allocate(i);
    }

    thread::sleep(Duration::from_millis(250));

    // Stop profiling
    let samples = profiler.stop();

    // Verify we got samples
    assert!(samples.len() > 0, "Expected memory samples");

    // Verify samples have reasonable timestamps
    let duration = profiler.duration();
    assert!(duration >= Duration::from_millis(500), "Expected at least 500ms duration");

    // Print some stats for manual verification
    println!("Memory profiling completed:");
    println!("  Duration: {:?}", duration);
    println!("  Samples: {}", samples.len());
    println!("  Average sample interval: {:?}",
             duration / samples.len() as u32);
}

#[test]
fn test_symbol_interning_performance() {
    let mut interner = SymbolInterner::new();

    let symbols = ["BTC", "ETH", "SOL", "ADA", "XRP", "DOGE", "DOT", "UNI", "LTC", "LINK"];

    // Intern symbols multiple times to test caching
    let start = std::time::Instant::now();

    for _ in 0..1000 {
        for symbol in &symbols {
            let _id = interner.intern_symbol(symbol);
        }
    }

    let duration = start.elapsed();

    // Verify performance
    assert!(duration < Duration::from_millis(100), "Symbol interning should be fast");

    // Verify statistics
    let stats = interner.stats();
    assert_eq!(stats.total_interns, 10000); // 10 symbols * 1000 iterations
    assert!(stats.cache_hits > 0, "Should have cache hits");
    assert!(stats.cache_misses > 0, "Should have cache misses for first intern");
    assert_eq!(stats.unique_strings, symbols.len(), "Should have correct unique string count");

    println!("Symbol interning performance test:");
    println!("  Duration: {:?}", duration);
    println!("  Total intern operations: {}", stats.total_interns);
    println!("  Cache hits: {}", stats.cache_hits);
    println!("  Cache misses: {}", stats.cache_misses);
    println!("  Cache hit rate: {:.2}%",
             (stats.cache_hits as f64 / stats.total_interns as f64) * 100.0);
}

#[test]
fn test_arena_allocator_performance() {
    let mut allocator = ArenaAllocator::new(64 * 1024);

    let start = std::time::Instant::now();

    // Allocate many objects
    let mut ptrs = Vec::new();
    for i in 0..10000 {
        let order = OptimizedOrder {
            symbol_id: SymbolId::from_raw(i % 10),
            price: 100.0 + i as f64,
            size: 1.0,
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            metadata: None,
        };
        let ptr = allocator.allocate(order);
        ptrs.push(ptr);
    }

    let allocation_duration = start.elapsed();

    // Reset and reallocate to test reset performance
    let reset_start = std::time::Instant::now();
    allocator.reset();
    let reset_duration = reset_start.elapsed();

    // Verify performance
    assert!(allocation_duration < Duration::from_millis(100), "Allocation should be fast");
    assert!(reset_duration < Duration::from_millis(10), "Reset should be very fast");

    // Verify statistics
    let stats = allocator.get_stats();
    assert_eq!(stats.total_allocations, 10000, "Should track allocations");
    assert!(stats.arena_allocations > 0, "Should have arena allocations");

    println!("Arena allocator performance test:");
    println!("  Allocation duration: {:?}", allocation_duration);
    println!("  Reset duration: {:?}", reset_duration);
    println!("  Total allocations: {}", stats.total_allocations);
    println!("  Arena allocations: {}", stats.arena_allocations);
}

#[test]
fn test_object_pool_performance() {
    let pool: ObjectPool<OptimizedOrder> = ObjectPool::new(100);

    let start = std::time::Instant::now();

    // Allocate and release objects multiple times
    for _ in 0..1000 {
        for _ in 0..50 {
            let mut order = pool.get();
            order.symbol_id = SymbolId::from_raw(0);
            order.price = 100.0;
            order.size = 1.0;
            order.side = OrderSide::Buy;
            order.order_type = OrderType::Limit;
            // Object automatically returned to pool when dropped
        }
    }

    let duration = start.elapsed();

    // Verify performance
    assert!(duration < Duration::from_millis(100), "Object pooling should be fast");

    // Verify statistics
    let stats = pool.get_stats();
    assert!(stats.total_allocations > 0, "Should have allocations");
    assert!(stats.pool_hits > 0, "Should have pool hits");
    assert!(stats.pool_misses > 0, "Should have initial pool misses");

    println!("Object pool performance test:");
    println!("  Duration: {:?}", duration);
    println!("  Total allocations: {}", stats.total_allocations);
    println!("  Pool hits: {}", stats.pool_hits);
    println!("  Pool misses: {}", stats.pool_misses);
    println!("  Pool hit rate: {:.2}%",
             (stats.pool_hits as f64 / stats.total_allocations as f64) * 100.0);
}

#[test]
fn test_trading_allocator_integration() {
    let mut allocator = TradingAllocator::new();

    let start = std::time::Instant::now();

    // Simulate high-frequency trading operations
    for i in 0..1000 {
        // Intern symbols
        let btc_id = allocator.intern_symbol("BTC");
        let eth_id = allocator.intern_symbol("ETH");

        // Allocate orders
        let _btc_order = allocator.allocate_order(
            "BTC",
            50000.0 + i as f64,
            0.1,
            OrderSide::Buy,
            OrderType::Limit,
        );
        let _eth_order = allocator.allocate_order(
            "ETH",
            3000.0 + i as f64,
            1.0,
            OrderSide::Sell,
            OrderType::Market,
        );

        // Get pooled objects
        let _pooled_order = allocator.pools.get_order();
        let _pooled_position = allocator.pools.get_position();

        // Reset arena periodically (simulating batch processing)
        if i % 100 == 0 {
            allocator.reset_arena();
        }
    }

    let duration = start.elapsed();

    // Verify performance
    assert!(duration < Duration::from_millis(200), "Trading allocator should be fast");

    // Verify statistics
    let stats = allocator.stats();
    assert!(stats.symbols.total_interns > 0, "Should have symbol intern operations");
    assert!(stats.pools.orders.total_allocations > 0, "Should have pooled allocations");
    assert!(stats.arena.total_allocations > 0, "Should have arena allocations");

    println!("Trading allocator integration test:");
    println!("  Duration: {:?}", duration);
    println!("  Symbol intern operations: {}", stats.symbols.total_interns);
    println!("  Symbol cache hit rate: {:.2}%",
             (stats.symbols.cache_hits as f64 / stats.symbols.total_interns as f64) * 100.0);
    println!("  Arena allocations: {}", stats.arena.total_allocations);
    println!("  Pooled orders: {}", stats.pools.orders.total_allocations);
}

#[test]
fn test_memory_usage_under_load() {
    let mut allocator = TradingAllocator::new();

    // Start memory profiling
    allocator.start_profiling();

    let start = std::time::Instant::now();

    // Simulate sustained high-load trading
    for batch in 0..10 {
        // Process a batch of operations
        for i in 0..1000 {
            let symbol = if i % 2 == 0 { "BTC" } else { "ETH" };
            let _order = allocator.allocate_order(
                symbol,
                1000.0 + i as f64,
                0.1,
                OrderSide::Buy,
                OrderType::Limit,
            );
        }

        // Reset arena between batches
        allocator.reset_arena();
        thread::sleep(Duration::from_millis(10));
    }

    let duration = start.elapsed();

    // Stop profiling and get samples
    let samples = allocator.stop_profiling();

    // Verify performance and memory behavior
    assert!(duration < Duration::from_secs(5), "Should complete within reasonable time");
    assert!(samples.len() > 0, "Should have memory samples");

    // Verify memory usage is reasonable (should not grow unboundedly)
    let first_sample = &samples[0];
    let last_sample = &samples[samples.len() - 1];

    println!("Memory usage under load test:");
    println!("  Duration: {:?}", duration);
    println!("  Samples collected: {}", samples.len());
    println!("  Initial allocated bytes: {}", first_sample.allocated_bytes);
    println!("  Final allocated bytes: {}", last_sample.allocated_bytes);
    println!("  Peak allocated bytes: {}", samples.iter().map(|s| s.peak_bytes).max().unwrap_or(0));

    // Memory should not grow unboundedly with arena resets
    assert!(last_sample.allocated_bytes < first_sample.allocated_bytes * 10,
            "Memory usage should not grow excessively");
}

#[test]
fn test_concurrent_memory_operations() {
    let interner = Arc::new(std::sync::Mutex::new(SymbolInterner::new()));
    let allocator = Arc::new(std::sync::Mutex::new(ArenaAllocator::new(64 * 1024)));

    let mut handles = vec![];

    // Spawn multiple threads performing memory operations
    for thread_id in 0..4 {
        let interner = Arc::clone(&interner);
        let allocator = Arc::clone(&allocator);

        let handle = thread::spawn(move || {
            for i in 0..1000 {
                // Intern symbols
                let symbol = if i % 2 == 0 { "BTC" } else { "ETH" };
                let id = {
                    let mut interner = interner.lock().unwrap();
                    interner.intern(symbol)
                };

                // Allocate objects
                {
                    let mut allocator = allocator.lock().unwrap();
                    let order = OptimizedOrder {
                        symbol_id: SymbolId::from_raw(id.raw() + thread_id),
                        price: 100.0 + i as f64,
                        size: 1.0,
                        side: OrderSide::Buy,
                        order_type: OrderType::Limit,
                        metadata: None,
                    };
                    let _ptr = allocator.allocate(order);
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify results
    let interner = interner.lock().unwrap();
    let stats = interner.stats();

    println!("Concurrent memory operations test:");
    println!("  Total intern operations: {}", stats.total_interns);
    println!("  Cache hits: {}", stats.cache_hits);
    println!("  Cache misses: {}", stats.cache_misses);
    println!("  Unique strings: {}", stats.unique_strings);

    // Should have some cache hits due to repeated symbol usage
    assert!(stats.cache_hits > 0, "Should have cache hits from concurrent usage");
}

#[test]
fn test_memory_optimization_memory_savings() {
    // Test that our optimizations actually save memory

    // Simulate storing many orders with string symbols
    let mut regular_orders = Vec::new();
    for i in 0..1000 {
        let symbol = if i % 2 == 0 { "BTC" } else { "ETH" };
        regular_orders.push((
            symbol.to_string(), // 4 bytes for "BTC" + allocation overhead
            100.0 + i as f64,
            1.0,
        ));
    }

    // Calculate approximate memory usage for regular approach
    let regular_memory: usize = regular_orders.iter()
        .map(|(symbol, _, _)| std::mem::size_of::<String>() + symbol.len())
        .sum();

    // Simulate optimized approach
    let mut interner = SymbolInterner::new();
    let mut optimized_orders = Vec::new();
    for i in 0..1000 {
        let symbol = if i % 2 == 0 { "BTC" } else { "ETH" };
        let symbol_id = interner.intern_symbol(symbol);
        optimized_orders.push((
            symbol_id, // 4 bytes
            100.0 + i as f64,
            1.0,
        ));
    }

    // Calculate approximate memory usage for optimized approach
    let symbol_overhead = interner.stats().unique_strings * 4; // 4 bytes per unique symbol
    let optimized_memory = optimized_orders.len() * std::mem::size_of::<(SymbolId, f64, f64)>()
                          + symbol_overhead;

    println!("Memory savings test:");
    println!("  Regular memory usage: {} bytes", regular_memory);
    println!("  Optimized memory usage: {} bytes", optimized_memory);
    println!("  Memory savings: {} bytes ({:.1}%)",
             regular_memory - optimized_memory,
             ((regular_memory - optimized_memory) as f64 / regular_memory as f64) * 100.0);

    // Should save significant memory
    assert!(regular_memory > optimized_memory, "Optimized approach should use less memory");
    assert!((regular_memory - optimized_memory) > 1000, "Should save at least 1KB");
}