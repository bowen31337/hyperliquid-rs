//! TWAP (Time-Weighted Average Price) execution types
//!
//! This module defines types for TWAP order execution tracking and slice fill reporting.

use serde::{Deserialize, Serialize};

/// TWAP slice fill execution record
/// Represents a single execution within a TWAP order slice
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwapSliceFill {
    /// Coin being traded
    pub coin: String,
    /// Side of the fill (Buy/Sell)
    pub side: String,
    /// Price at which the fill occurred
    pub px: String,
    /// Size of the fill
    pub sz: String,
    /// Timestamp of the fill
    pub time: i64,
    /// Transaction hash (optional)
    pub hash: Option<String>,
    /// Fee paid for the fill
    pub fee: Option<String>,
    /// Fee asset (optional)
    pub fee_asset: Option<String>,
    /// Order ID that generated this fill
    pub oid: Option<i64>,
    /// Slice ID for TWAP order
    pub slice_id: String,
    /// Slice number within the TWAP order
    pub slice_number: i64,
    /// Total slices in the TWAP order
    pub total_slices: i64,
    /// Execution status of this slice
    pub slice_status: String,
    /// Target price for this slice (if specified)
    pub target_px: Option<String>,
    /// Deviation from target price (percentage)
    pub price_deviation: Option<String>,
}

/// TWAP slice information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwapSlice {
    /// Slice ID
    pub slice_id: String,
    /// Slice number
    pub slice_number: i64,
    /// Total number of slices in the TWAP order
    pub total_slices: i64,
    /// Target size for this slice
    pub target_sz: String,
    /// Executed size for this slice
    pub executed_sz: String,
    /// Target price for this slice (if specified)
    pub target_px: Option<String>,
    /// Average execution price for this slice
    pub avg_px: String,
    /// Execution status (pending, partially_filled, filled, cancelled)
    pub status: String,
    /// Start time of slice execution
    pub start_time: i64,
    /// End time of slice execution (if completed)
    pub end_time: Option<i64>,
    /// List of fills for this slice
    pub fills: Vec<TwapSliceFill>,
    /// Total fees paid for this slice
    pub total_fees: Option<String>,
}

/// TWAP order execution summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwapExecutionSummary {
    /// TWAP order ID
    pub twap_order_id: String,
    /// User address
    pub user: String,
    /// Coin being traded
    pub coin: String,
    /// Total target size for the TWAP order
    pub total_target_sz: String,
    /// Total executed size across all slices
    pub total_executed_sz: String,
    /// Average execution price across all slices
    pub avg_px: String,
    /// Overall execution status
    pub status: String,
    /// Start time of TWAP order
    pub start_time: i64,
    /// End time of TWAP order (if completed)
    pub end_time: Option<i64>,
    /// Total number of slices
    pub total_slices: i64,
    /// Number of completed slices
    pub completed_slices: i64,
    /// Number of failed slices
    pub failed_slices: i64,
    /// Total fees paid across all slices
    pub total_fees: Option<String>,
    /// Price deviation from target (percentage)
    pub price_deviation: Option<String>,
    /// Execution quality metrics
    pub execution_quality: Option<ExecutionQuality>,
}

/// Execution quality metrics for TWAP orders
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionQuality {
    /// Time-weighted average price deviation
    pub twap_deviation: String,
    /// Volume-weighted average price deviation
    pub vwap_deviation: String,
    /// Maximum price deviation during execution
    pub max_deviation: String,
    /// Slippage compared to start price
    pub slippage: String,
    /// Market impact percentage
    pub market_impact: String,
    /// Execution efficiency score (0-100)
    pub efficiency_score: i32,
}

/// TWAP slice fills response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwapSliceFillsResponse {
    /// TWAP order ID
    pub twap_order_id: String,
    /// User address
    pub user: String,
    /// Coin being traded
    pub coin: String,
    /// Execution summary
    pub execution_summary: TwapExecutionSummary,
    /// Individual slice details
    pub slices: Vec<TwapSlice>,
    /// All fills across all slices
    pub all_fills: Vec<TwapSliceFill>,
    /// Request timestamp
    pub timestamp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twap_slice_fill_creation() {
        let fill = TwapSliceFill {
            coin: "BTC".to_string(),
            side: "Buy".to_string(),
            px: "50000.0".to_string(),
            sz: "0.01".to_string(),
            time: 1640995200000,
            hash: Some("0x1234567890abcdef".to_string()),
            fee: Some("0.0001".to_string()),
            fee_asset: Some("USDC".to_string()),
            oid: Some(12345),
            slice_id: "slice_001".to_string(),
            slice_number: 1,
            total_slices: 10,
            slice_status: "filled".to_string(),
            target_px: Some("50000.0".to_string()),
            price_deviation: Some("0.0".to_string()),
        };

        assert_eq!(fill.coin, "BTC");
        assert_eq!(fill.side, "Buy");
        assert_eq!(fill.slice_number, 1);
        assert_eq!(fill.total_slices, 10);
        assert!(fill.hash.is_some());
    }

    #[test]
    fn test_twap_slice_fill_serialization() {
        let fill = TwapSliceFill {
            coin: "ETH".to_string(),
            side: "Sell".to_string(),
            px: "3000.0".to_string(),
            sz: "0.5".to_string(),
            time: 1640995200000,
            hash: None,
            fee: None,
            fee_asset: None,
            oid: None,
            slice_id: "slice_002".to_string(),
            slice_number: 2,
            total_slices: 10,
            slice_status: "partially_filled".to_string(),
            target_px: None,
            price_deviation: None,
        };

        let json = serde_json::to_string(&fill).unwrap();
        let expected = r#"{"coin":"ETH","side":"Sell","px":"3000.0","sz":"0.5","time":1640995200000,"sliceId":"slice_002","sliceNumber":2,"totalSlices":10,"sliceStatus":"partially_filled"}"#;
        assert_eq!(json, expected);

        let deserialized: TwapSliceFill = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.coin, "ETH");
        assert_eq!(deserialized.side, "Sell");
    }

    #[test]
    fn test_twap_slice_creation() {
        let slice = TwapSlice {
            slice_id: "slice_001".to_string(),
            slice_number: 1,
            total_slices: 10,
            target_sz: "0.1".to_string(),
            executed_sz: "0.1".to_string(),
            target_px: Some("50000.0".to_string()),
            avg_px: "50000.0".to_string(),
            status: "filled".to_string(),
            start_time: 1640995200000,
            end_time: Some(1640995260000),
            fills: vec![],
            total_fees: Some("0.0001".to_string()),
        };

        assert_eq!(slice.slice_number, 1);
        assert_eq!(slice.total_slices, 10);
        assert_eq!(slice.status, "filled");
        assert!(slice.total_fees.is_some());
    }

    #[test]
    fn test_twap_execution_summary_creation() {
        let summary = TwapExecutionSummary {
            twap_order_id: "twap_12345".to_string(),
            user: "0x1234567890abcdef".to_string(),
            coin: "BTC".to_string(),
            total_target_sz: "1.0".to_string(),
            total_executed_sz: "1.0".to_string(),
            avg_px: "50000.0".to_string(),
            status: "completed".to_string(),
            start_time: 1640995200000,
            end_time: Some(1640998800000),
            total_slices: 10,
            completed_slices: 10,
            failed_slices: 0,
            total_fees: Some("0.001".to_string()),
            price_deviation: Some("0.5".to_string()),
            execution_quality: Some(ExecutionQuality {
                twap_deviation: "0.1".to_string(),
                vwap_deviation: "0.2".to_string(),
                max_deviation: "0.8".to_string(),
                slippage: "0.05".to_string(),
                market_impact: "0.02".to_string(),
                efficiency_score: 95,
            }),
        };

        assert_eq!(summary.twap_order_id, "twap_12345");
        assert_eq!(summary.total_slices, 10);
        assert_eq!(summary.completed_slices, 10);
        assert!(summary.execution_quality.is_some());
    }

    #[test]
    fn test_twap_slice_fills_response_creation() {
        let response = TwapSliceFillsResponse {
            twap_order_id: "twap_12345".to_string(),
            user: "0x1234567890abcdef".to_string(),
            coin: "BTC".to_string(),
            execution_summary: TwapExecutionSummary {
                twap_order_id: "twap_12345".to_string(),
                user: "0x1234567890abcdef".to_string(),
                coin: "BTC".to_string(),
                total_target_sz: "1.0".to_string(),
                total_executed_sz: "1.0".to_string(),
                avg_px: "50000.0".to_string(),
                status: "completed".to_string(),
                start_time: 1640995200000,
                end_time: Some(1640998800000),
                total_slices: 10,
                completed_slices: 10,
                failed_slices: 0,
                total_fees: Some("0.001".to_string()),
                price_deviation: Some("0.5".to_string()),
                execution_quality: None,
            },
            slices: vec![],
            all_fills: vec![],
            timestamp: 1640998800000,
        };

        assert_eq!(response.twap_order_id, "twap_12345");
        assert_eq!(response.coin, "BTC");
        assert_eq!(response.slices.len(), 0);
        assert_eq!(response.all_fills.len(), 0);
    }

    #[test]
    fn test_twap_response_serialization() {
        let response = TwapSliceFillsResponse {
            twap_order_id: "twap_12345".to_string(),
            user: "0x1234567890abcdef".to_string(),
            coin: "BTC".to_string(),
            execution_summary: TwapExecutionSummary {
                twap_order_id: "twap_12345".to_string(),
                user: "0x1234567890abcdef".to_string(),
                coin: "BTC".to_string(),
                total_target_sz: "1.0".to_string(),
                total_executed_sz: "1.0".to_string(),
                avg_px: "50000.0".to_string(),
                status: "completed".to_string(),
                start_time: 1640995200000,
                end_time: Some(1640998800000),
                total_slices: 10,
                completed_slices: 10,
                failed_slices: 0,
                total_fees: Some("0.001".to_string()),
                price_deviation: Some("0.5".to_string()),
                execution_quality: None,
            },
            slices: vec![],
            all_fills: vec![],
            timestamp: 1640998800000,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: TwapSliceFillsResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.twap_order_id, "twap_12345");
        assert_eq!(deserialized.user, "0x1234567890abcdef");
        assert_eq!(deserialized.execution_summary.total_slices, 10);
    }
}