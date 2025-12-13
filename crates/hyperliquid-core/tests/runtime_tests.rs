//! Tests for Tokio async runtime configuration

use hyperliquid_core::runtime::{
    RuntimeConfig, ConfiguredRuntime,
    create_default_runtime, create_high_throughput_runtime,
    create_low_latency_runtime, create_single_threaded_runtime,
};
use std::time::Duration;
use tokio::time;

/// Test basic runtime configuration
#[test]
fn test_runtime_config_default() {
    let config = RuntimeConfig::default();

    // Verify default values
    let num_cpus = num_cpus::get();
    assert_eq!(config.worker_threads, num_cpus);
    assert_eq!(config.max_blocking_threads, 512);
    assert_eq!(config.thread_stack_size, 2 * 1024 * 1024);
    assert!(config.enable_io);
    assert!(config.enable_time);
    assert_eq!(config.global_queue_interval, 61);
    assert_eq!(config.shutdown_timeout_secs, 30);
}

/// Test custom runtime configuration
#[test]
fn test_runtime_config_custom() {
    let config = RuntimeConfig::new(4, 128, 1 * 1024 * 1024);

    assert_eq!(config.worker_threads, 4);
    assert_eq!(config.max_blocking_threads, 128);
    assert_eq!(config.thread_stack_size, 1 * 1024 * 1024);
    assert!(config.enable_io);
    assert!(config.enable_time);
}

/// Test high-throughput configuration
#[test]
fn test_runtime_config_high_throughput() {
    let config = RuntimeConfig::high_throughput();
    let num_cpus = num_cpus::get();

    assert_eq!(config.worker_threads, num_cpus * 2);
    assert_eq!(config.max_blocking_threads, 1024);
    assert_eq!(config.thread_stack_size, 4 * 1024 * 1024);
    assert_eq!(config.global_queue_interval, 31);
}

/// Test low-latency configuration
#[test]
fn test_runtime_config_low_latency() {
    let config = RuntimeConfig::low_latency();
    let num_cpus = num_cpus::get();

    assert_eq!(config.worker_threads, num_cpus);
    assert_eq!(config.max_blocking_threads, 256);
    assert_eq!(config.thread_stack_size, 1 * 1024 * 1024);
    assert_eq!(config.global_queue_interval, 127);
}

/// Test single-threaded configuration
#[test]
fn test_runtime_config_single_threaded() {
    let config = RuntimeConfig::single_threaded();

    assert_eq!(config.worker_threads, 1);
    assert_eq!(config.max_blocking_threads, 128);
    assert_eq!(config.thread_stack_size, 1 * 1024 * 1024);
}

/// Test runtime creation with default configuration
#[test]
fn test_create_default_runtime() {
    let runtime = create_default_runtime();
    assert!(runtime.is_ok());

    let runtime = runtime.unwrap();
    assert!(runtime.is_running());

    // Verify configuration
    let config = runtime.config();
    let num_cpus = num_cpus::get();
    assert_eq!(config.worker_threads, num_cpus);
    assert_eq!(config.max_blocking_threads, 512);
}

/// Test runtime creation with high-throughput configuration
#[test]
fn test_create_high_throughput_runtime() {
    let runtime = create_high_throughput_runtime();
    assert!(runtime.is_ok());

    let runtime = runtime.unwrap();
    assert!(runtime.is_running());

    let config = runtime.config();
    let num_cpus = num_cpus::get();
    assert_eq!(config.worker_threads, num_cpus * 2);
}

/// Test runtime creation with low-latency configuration
#[test]
fn test_create_low_latency_runtime() {
    let runtime = create_low_latency_runtime();
    assert!(runtime.is_ok());

    let runtime = runtime.unwrap();
    assert!(runtime.is_running());

    let config = runtime.config();
    let num_cpus = num_cpus::get();
    assert_eq!(config.worker_threads, num_cpus);
}

/// Test runtime creation with single-threaded configuration
#[test]
fn test_create_single_threaded_runtime() {
    let runtime = create_single_threaded_runtime();
    assert!(runtime.is_ok());

    let runtime = runtime.unwrap();
    assert!(runtime.is_running());

    let config = runtime.config();
    assert_eq!(config.worker_threads, 1);
}

/// Test runtime shutdown
#[test]
fn test_runtime_shutdown() {
    let runtime = create_default_runtime().unwrap();
    assert!(runtime.is_running());

    // Shutdown should succeed
    let result = runtime.shutdown();
    assert!(result.is_ok());
}

/// Test runtime block_on functionality
#[test]
fn test_runtime_block_on() {
    let runtime = create_default_runtime().unwrap();

    // Test blocking on a simple future
    let result = runtime.block_on(async {
        time::sleep(Duration::from_millis(10)).await;
        42
    });

    assert_eq!(result, 42);
}

/// Test runtime with async task execution
#[test]
fn test_runtime_async_tasks() {
    let runtime = create_default_runtime().unwrap();

    let result = runtime.block_on(async {
        let task1 = tokio::spawn(async {
            time::sleep(Duration::from_millis(10)).await;
            1
        });

        let task2 = tokio::spawn(async {
            time::sleep(Duration::from_millis(20)).await;
            2
        });

        let result1 = task1.await.unwrap();
        let result2 = task2.await.unwrap();

        result1 + result2
    });

    assert_eq!(result, 3);
}

/// Test runtime configuration with invalid parameters
#[test]
fn test_runtime_invalid_configuration() {
    // Test with zero worker threads (should use default)
    let config = RuntimeConfig {
        worker_threads: 0,
        ..Default::default()
    };

    // This should still create a runtime (will use default worker threads)
    let runtime = ConfiguredRuntime::new(config);
    assert!(runtime.is_ok());
}

/// Test runtime drop without explicit shutdown
#[test]
fn test_runtime_drop_without_shutdown() {
    let runtime = create_default_runtime().unwrap();
    assert!(runtime.is_running());

    // Runtime should be dropped here, triggering implicit shutdown
    drop(runtime);
}

/// Test runtime configuration cloning
#[test]
fn test_runtime_config_clone() {
    let config1 = RuntimeConfig::high_throughput();
    let config2 = config1.clone();

    assert_eq!(config1.worker_threads, config2.worker_threads);
    assert_eq!(config1.max_blocking_threads, config2.max_blocking_threads);
    assert_eq!(config1.thread_stack_size, config2.thread_stack_size);
}

/// Test runtime configuration debug formatting
#[test]
fn test_runtime_config_debug() {
    let config = RuntimeConfig::default();
    let debug_output = format!("{:?}", config);

    // Verify debug output contains expected fields
    assert!(debug_output.contains("worker_threads"));
    assert!(debug_output.contains("max_blocking_threads"));
    assert!(debug_output.contains("thread_stack_size"));
}

/// Test runtime with multiple concurrent tasks
#[test]
fn test_runtime_concurrent_tasks() {
    let runtime = create_default_runtime().unwrap();

    let result = runtime.block_on(async {
        let mut handles = vec![];

        for i in 0..10 {
            let handle = tokio::spawn(async move {
                time::sleep(Duration::from_millis(i * 10)).await;
                i * 2
            });
            handles.push(handle);
        }

        let mut results = vec![];
        for handle in handles {
            results.push(handle.await.unwrap());
        }

        results.iter().sum::<i32>()
    });

    // Sum of 0 + 2 + 4 + ... + 18 = 90
    assert_eq!(result, 90);
}

/// Test runtime graceful shutdown with timeout
#[test]
fn test_runtime_graceful_shutdown() {
    let runtime = create_default_runtime().unwrap();

    // Spawn a long-running task
    runtime.block_on(async {
        tokio::spawn(async {
            time::sleep(Duration::from_secs(5)).await;
        });
    });

    // Shutdown should complete (task will be cancelled)
    let start = std::time::Instant::now();
    let result = runtime.shutdown();
    let duration = start.elapsed();

    assert!(result.is_ok());
    // Shutdown should complete within timeout (30 seconds)
    assert!(duration < Duration::from_secs(30));
}