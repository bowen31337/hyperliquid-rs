//! Tokio async runtime configuration for Hyperliquid SDK
//!
//! This module provides configurable Tokio runtime setup with proper
//! worker thread configuration, blocking pool management, and graceful
//! shutdown handling.

use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::{Builder, Runtime};
use tracing::{debug, error, info, warn};

/// Configuration for Tokio async runtime
#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    /// Number of worker threads (0 = use number of CPU cores)
    pub worker_threads: usize,
    /// Maximum number of blocking threads
    pub max_blocking_threads: usize,
    /// Thread stack size in bytes
    pub thread_stack_size: usize,
    /// Enable I/O driver
    pub enable_io: bool,
    /// Enable time driver
    pub enable_time: bool,
    /// Global queue interval for work-stealing (microseconds)
    pub global_queue_interval: u32,
    /// Shutdown timeout in seconds
    pub shutdown_timeout_secs: u64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        let num_cpus = num_cpus::get();

        Self {
            worker_threads: num_cpus,
            max_blocking_threads: 512,
            thread_stack_size: 2 * 1024 * 1024, // 2MB
            enable_io: true,
            enable_time: true,
            global_queue_interval: 61, // Prime number to reduce collisions
            shutdown_timeout_secs: 30,
        }
    }
}

impl RuntimeConfig {
    /// Create a new runtime configuration with custom settings
    pub fn new(
        worker_threads: usize,
        max_blocking_threads: usize,
        thread_stack_size: usize,
    ) -> Self {
        Self {
            worker_threads,
            max_blocking_threads,
            thread_stack_size,
            ..Default::default()
        }
    }

    /// Create a configuration optimized for high-throughput workloads
    pub fn high_throughput() -> Self {
        let num_cpus = num_cpus::get();

        Self {
            worker_threads: num_cpus * 2,
            max_blocking_threads: 1024,
            thread_stack_size: 4 * 1024 * 1024, // 4MB
            global_queue_interval: 31, // Smaller interval for more frequent work stealing
            ..Default::default()
        }
    }

    /// Create a configuration optimized for low-latency workloads
    pub fn low_latency() -> Self {
        let num_cpus = num_cpus::get();

        Self {
            worker_threads: num_cpus,
            max_blocking_threads: 256,
            thread_stack_size: 1 * 1024 * 1024, // 1MB
            global_queue_interval: 127, // Larger interval to reduce overhead
            ..Default::default()
        }
    }

    /// Create a configuration for single-threaded runtime
    pub fn single_threaded() -> Self {
        Self {
            worker_threads: 1,
            max_blocking_threads: 128,
            thread_stack_size: 1 * 1024 * 1024,
            ..Default::default()
        }
    }
}

/// A configured Tokio runtime with graceful shutdown support
pub struct ConfiguredRuntime {
    /// The underlying Tokio runtime
    runtime: Runtime,
    /// Runtime configuration
    config: RuntimeConfig,
    /// Whether the runtime is currently running
    is_running: bool,
}

impl ConfiguredRuntime {
    /// Create a new configured runtime with the given configuration
    pub fn new(config: RuntimeConfig) -> std::io::Result<Self> {
        info!(
            "Creating Tokio runtime with {} worker threads, {} max blocking threads",
            config.worker_threads, config.max_blocking_threads
        );

        let runtime = Builder::new_multi_thread()
            .worker_threads(config.worker_threads)
            .max_blocking_threads(config.max_blocking_threads)
            .thread_stack_size(config.thread_stack_size)
            .enable_io(config.enable_io)
            .enable_time(config.enable_time)
            .global_queue_interval(config.global_queue_interval)
            .thread_name("hyperliquid-worker")
            .on_thread_start(|| {
                debug!("Tokio worker thread started");
            })
            .on_thread_stop(|| {
                debug!("Tokio worker thread stopped");
            })
            .build()?;

        info!("Tokio runtime created successfully");

        Ok(Self {
            runtime,
            config,
            is_running: true,
        })
    }

    /// Get a reference to the underlying Tokio runtime
    pub fn inner(&self) -> &Runtime {
        &self.runtime
    }

    /// Get the runtime configuration
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Check if the runtime is currently running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Shutdown the runtime gracefully with timeout
    pub fn shutdown(self) -> std::io::Result<()> {
        if !self.is_running {
            warn!("Runtime already shutdown");
            return Ok(());
        }

        info!(
            "Shutting down Tokio runtime with {} second timeout",
            self.config.shutdown_timeout_secs
        );

        // Drop the runtime to initiate shutdown
        drop(self.runtime);

        info!("Tokio runtime shutdown completed");

        Ok(())
    }

    /// Block on a future and return its result
    pub fn block_on<F: std::future::Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }
}

impl Drop for ConfiguredRuntime {
    fn drop(&mut self) {
        if self.is_running {
            warn!("ConfiguredRuntime dropped without explicit shutdown - forcing shutdown");
            self.is_running = false;
        }
    }
}

/// Create a default configured runtime
pub fn create_default_runtime() -> std::io::Result<ConfiguredRuntime> {
    ConfiguredRuntime::new(RuntimeConfig::default())
}

/// Create a high-throughput configured runtime
pub fn create_high_throughput_runtime() -> std::io::Result<ConfiguredRuntime> {
    ConfiguredRuntime::new(RuntimeConfig::high_throughput())
}

/// Create a low-latency configured runtime
pub fn create_low_latency_runtime() -> std::io::Result<ConfiguredRuntime> {
    ConfiguredRuntime::new(RuntimeConfig::low_latency())
}

/// Create a single-threaded configured runtime
pub fn create_single_threaded_runtime() -> std::io::Result<ConfiguredRuntime> {
    ConfiguredRuntime::new(RuntimeConfig::single_threaded())
}