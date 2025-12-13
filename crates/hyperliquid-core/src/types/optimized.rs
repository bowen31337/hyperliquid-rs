//! Optimized types with memory allocation improvements
//!
//! This module provides optimized versions of common types with memory
//! allocation improvements including string interning for symbols.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::memory::{StringInterner, ZeroCopyValue};
use super::OrderType;

/// Symbol intern pool for trading pairs (BTC, ETH, etc.)
/// Provides efficient string deduplication for frequently used symbols
#[derive(Debug, Clone)]
pub struct SymbolInterner {
    interner: StringInterner,
}

impl SymbolInterner {
    /// Create a new symbol interner
    pub fn new() -> Self {
        Self {
            interner: StringInterner::new(),
        }
    }

    /// Intern a trading symbol
    pub fn intern_symbol(&mut self, symbol: &str) -> SymbolId {
        SymbolId(self.interner.intern(symbol))
    }

    /// Get symbol from ID
    pub fn get_symbol(&self, id: SymbolId) -> Option<&str> {
        self.interner.get(id.0)
    }

    /// Get interner statistics
    pub fn stats(&self) -> crate::memory::StringInternStats {
        self.interner.get_stats()
    }
}

/// Interned symbol ID (4 bytes instead of variable-length string)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(u32);

impl SymbolId {
    /// Create a symbol ID from raw ID
    pub fn from_raw(id: u32) -> Self {
        Self(id)
    }

    /// Get raw ID
    pub fn raw(&self) -> u32 {
        self.0
    }
}

/// Optimized order with interned symbols and zero-copy values
#[derive(Debug, Clone)]
pub struct OptimizedOrder {
    /// Interned symbol ID (4 bytes)
    pub symbol_id: SymbolId,
    /// Price as f64 (no string allocation)
    pub price: f64,
    /// Size as f64 (no string allocation)
    pub size: f64,
    /// Side (buy/sell) as enum (no string allocation)
    pub side: OrderSide,
    /// Order type as enum (no string allocation)
    pub order_type: OrderType,
    /// Optional zero-copy metadata
    pub metadata: Option<ZeroCopyValue<'static>>,
}

/// Order side (no string allocation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Order type (no string allocation)
pub type OrderType = super::OrderType;

/// Optimized position with interned symbols
#[derive(Debug, Clone)]
pub struct OptimizedPosition {
    /// Interned symbol ID
    pub symbol_id: SymbolId,
    /// Position size
    pub size: f64,
    /// Entry price
    pub entry_price: f64,
    /// Current mark price
    pub mark_price: f64,
    /// Unrealized PnL
    pub unrealized_pnl: f64,
    /// Realized PnL
    pub realized_pnl: f64,
}

/// Optimized L2 order book with interned symbols
#[derive(Debug, Clone)]
pub struct OptimizedL2Book {
    /// Interned symbol ID
    pub symbol_id: SymbolId,
    /// Timestamp
    pub timestamp: i64,
    /// Bids (price, size)
    pub bids: Vec<(f64, f64)>,
    /// Asks (price, size)
    pub asks: Vec<(f64, f64)>,
}

/// Optimized trade with interned symbols
#[derive(Debug, Clone)]
pub struct OptimizedTrade {
    /// Interned symbol ID
    pub symbol_id: SymbolId,
    /// Trade price
    pub price: f64,
    /// Trade size
    pub size: f64,
    /// Timestamp
    pub timestamp: i64,
    /// Side
    pub side: OrderSide,
}

/// Memory-optimized user state container
#[derive(Debug, Clone)]
pub struct OptimizedUserState {
    /// Interned address
    pub address: String, // Address is unique per user, so not interned
    /// Positions by symbol ID
    pub positions: HashMap<SymbolId, OptimizedPosition>,
    /// Collateral
    pub collateral: f64,
    /// Maintenance margin
    pub maintenance_margin: f64,
    /// Account value
    pub account_value: f64,
}

/// Memory pool for frequently allocated trading objects
pub struct TradingObjectPool {
    /// Order pool
    pub orders: crate::memory::ObjectPool<OptimizedOrder>,
    /// Position pool
    pub positions: crate::memory::ObjectPool<OptimizedPosition>,
    /// L2 book pool
    pub l2_books: crate::memory::ObjectPool<OptimizedL2Book>,
    /// Trade pool
    pub trades: crate::memory::ObjectPool<OptimizedTrade>,
}

impl TradingObjectPool {
    /// Create a new trading object pool
    pub fn new() -> Self {
        Self {
            orders: crate::memory::ObjectPool::new(1000),
            positions: crate::memory::ObjectPool::new(100),
            l2_books: crate::memory::ObjectPool::new(50),
            trades: crate::memory::ObjectPool::new(1000),
        }
    }

    /// Get an order from the pool
    pub fn get_order(&self) -> crate::memory::PooledObject<OptimizedOrder> {
        self.orders.get()
    }

    /// Get a position from the pool
    pub fn get_position(&self) -> crate::memory::PooledObject<OptimizedPosition> {
        self.positions.get()
    }

    /// Get an L2 book from the pool
    pub fn get_l2_book(&self) -> crate::memory::PooledObject<OptimizedL2Book> {
        self.l2_books.get()
    }

    /// Get a trade from the pool
    pub fn get_trade(&self) -> crate::memory::PooledObject<OptimizedTrade> {
        self.trades.get()
    }

    /// Get pool statistics
    pub fn stats(&self) -> TradingPoolStats {
        TradingPoolStats {
            orders: self.orders.get_stats(),
            positions: self.positions.get_stats(),
            l2_books: self.l2_books.get_stats(),
            trades: self.trades.get_stats(),
        }
    }
}

/// Statistics for trading object pools
#[derive(Debug, Clone)]
pub struct TradingPoolStats {
    pub orders: crate::memory::PoolStats,
    pub positions: crate::memory::PoolStats,
    pub l2_books: crate::memory::PoolStats,
    pub trades: crate::memory::PoolStats,
}

/// High-performance allocator for trading data
/// Combines arena allocation, string interning, and object pooling
pub struct TradingAllocator {
    /// Arena allocator for short-lived objects
    pub arena: crate::memory::ArenaAllocator,
    /// Symbol interner for trading pairs
    pub symbols: SymbolInterner,
    /// Object pools for frequently allocated types
    pub pools: TradingObjectPool,
    /// Memory profiler for monitoring
    pub profiler: crate::memory::MemoryProfiler,
}

impl TradingAllocator {
    /// Create a new trading allocator with default settings
    pub fn new() -> Self {
        Self {
            arena: crate::memory::ArenaAllocator::new(64 * 1024), // 64KB chunks
            symbols: SymbolInterner::new(),
            pools: TradingObjectPool::new(),
            profiler: crate::memory::MemoryProfiler::new(std::time::Duration::from_secs(1)),
        }
    }

    /// Intern a trading symbol
    pub fn intern_symbol(&mut self, symbol: &str) -> SymbolId {
        self.symbols.intern_symbol(symbol)
    }

    /// Allocate an order with memory optimization
    pub fn allocate_order(
        &mut self,
        symbol: &str,
        price: f64,
        size: f64,
        side: OrderSide,
        order_type: OrderType,
    ) -> *mut OptimizedOrder {
        let symbol_id = self.intern_symbol(symbol);
        let order = OptimizedOrder {
            symbol_id,
            price,
            size,
            side,
            order_type,
            metadata: None,
        };
        self.arena.allocate(order)
    }

    /// Allocate a position with memory optimization
    pub fn allocate_position(
        &mut self,
        symbol: &str,
        size: f64,
        entry_price: f64,
        mark_price: f64,
        unrealized_pnl: f64,
        realized_pnl: f64,
    ) -> *mut OptimizedPosition {
        let symbol_id = self.intern_symbol(symbol);
        let position = OptimizedPosition {
            symbol_id,
            size,
            entry_price,
            mark_price,
            unrealized_pnl,
            realized_pnl,
        };
        self.arena.allocate(position)
    }

    /// Get allocation statistics
    pub fn stats(&self) -> TradingAllocatorStats {
        TradingAllocatorStats {
            arena: self.arena.get_stats(),
            symbols: self.symbols.stats(),
            pools: self.pools.stats(),
        }
    }

    /// Reset arena allocator (fast clear for batch operations)
    pub fn reset_arena(&mut self) {
        self.arena.reset();
    }

    /// Start memory profiling
    pub fn start_profiling(&self) {
        self.profiler.start();
    }

    /// Stop memory profiling and get samples
    pub fn stop_profiling(&self) -> Vec<crate::memory::MemorySample> {
        self.profiler.stop()
    }
}

/// Combined statistics for trading allocator
#[derive(Debug, Clone)]
pub struct TradingAllocatorStats {
    pub arena: crate::memory::AllocationStats,
    pub symbols: crate::memory::StringInternStats,
    pub pools: TradingPoolStats,
}

impl Default for SymbolInterner {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TradingObjectPool {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TradingAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_interner() {
        let mut interner = SymbolInterner::new();

        let btc_id = interner.intern_symbol("BTC");
        let eth_id = interner.intern_symbol("ETH");
        let btc_id2 = interner.intern_symbol("BTC");

        assert_eq!(btc_id, btc_id2);
        assert_ne!(btc_id, eth_id);

        assert_eq!(interner.get_symbol(btc_id), Some("BTC"));
        assert_eq!(interner.get_symbol(eth_id), Some("ETH"));

        let stats = interner.stats();
        assert_eq!(stats.total_interns, 3);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 2);
    }

    #[test]
    fn test_optimized_order() {
        let order = OptimizedOrder {
            symbol_id: SymbolId::from_raw(0),
            price: 50000.0,
            size: 1.0,
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            metadata: None,
        };

        assert_eq!(order.price, 50000.0);
        assert_eq!(order.side, OrderSide::Buy);
        assert_eq!(order.order_type, OrderType::Limit);
    }

    #[test]
    fn test_trading_allocator() {
        let mut allocator = TradingAllocator::new();

        // Intern some symbols
        let btc_id = allocator.intern_symbol("BTC");
        let eth_id = allocator.intern_symbol("ETH");

        // Allocate orders
        let order_ptr = allocator.allocate_order("BTC", 50000.0, 1.0, OrderSide::Buy, OrderType::Limit);
        let eth_order_ptr = allocator.allocate_order("ETH", 3000.0, 2.0, OrderSide::Sell, OrderType::Market);

        assert!(!order_ptr.is_null());
        assert!(!eth_order_ptr.is_null());

        unsafe {
            let order = &*order_ptr;
            assert_eq!(order.symbol_id, btc_id);
            assert_eq!(order.price, 50000.0);
            assert_eq!(order.side, OrderSide::Buy);

            let eth_order = &*eth_order_ptr;
            assert_eq!(eth_order.symbol_id, eth_id);
            assert_eq!(eth_order.price, 3000.0);
            assert_eq!(eth_order.side, OrderSide::Sell);
        }

        // Test arena reset
        allocator.reset_arena();

        let stats = allocator.stats();
        assert_eq!(stats.symbols.total_interns, 2);
        assert_eq!(stats.symbols.cache_misses, 2);
    }

    #[test]
    fn test_trading_object_pool() {
        let pool = TradingObjectPool::new();

        // Get objects from pool
        let mut order = pool.get_order();
        order.symbol_id = SymbolId::from_raw(0);
        order.price = 100.0;
        order.size = 1.0;

        let mut position = pool.get_position();
        position.symbol_id = SymbolId::from_raw(1);
        position.size = 2.0;

        // Objects automatically returned to pool when dropped

        let stats = pool.stats();
        assert!(stats.orders.total_allocations > 0);
        assert!(stats.positions.total_allocations > 0);
    }
}