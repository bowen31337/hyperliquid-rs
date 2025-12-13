//! Tests for MarginSummary calculations

use hyperliquid_core::types::MarginSummary;

#[tokio::test]
async fn test_margin_summary_creation() {
    let summary = MarginSummary::new(
        "10000.0".to_string(),
        "2000.0".to_string(),
        "5000.0".to_string(),
        "8000.0".to_string(),
    );

    assert_eq!(summary.account_value(), "10000.0");
    assert_eq!(summary.total_margin_used(), "2000.0");
    assert_eq!(summary.total_ntl_pos(), "5000.0");
    assert_eq!(summary.total_raw_usd(), "8000.0");
}

#[tokio::test]
async fn test_margin_utilization() {
    let summary = MarginSummary::new(
        "10000.0".to_string(),
        "2000.0".to_string(),
        "5000.0".to_string(),
        "8000.0".to_string(),
    );

    let utilization = summary.margin_utilization().unwrap();
    assert_eq!(utilization, 20.0); // 20% utilization

    // Test edge case: zero account value
    let summary_zero = MarginSummary::new(
        "0.0".to_string(),
        "1000.0".to_string(),
        "5000.0".to_string(),
        "8000.0".to_string(),
    );

    let result = summary_zero.margin_utilization();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Account value cannot be zero"));
}

#[tokio::test]
async fn test_maintenance_margin() {
    let summary = MarginSummary::new(
        "10000.0".to_string(),
        "2000.0".to_string(),
        "5000.0".to_string(),
        "8000.0".to_string(),
    );

    let maintenance_margin = summary.maintenance_margin().unwrap();
    assert_eq!(maintenance_margin, 1.6); // 8000 / 5000 = 1.6

    // Test edge case: zero total NTL position
    let summary_zero = MarginSummary::new(
        "10000.0".to_string(),
        "2000.0".to_string(),
        "0.0".to_string(),
        "8000.0".to_string(),
    );

    let result = summary_zero.maintenance_margin();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Total NTL position cannot be zero"));
}

#[tokio::test]
async fn test_liquidation_threshold() {
    let summary = MarginSummary::new(
        "10000.0".to_string(),
        "2000.0".to_string(),
        "5000.0".to_string(),
        "8000.0".to_string(),
    );

    // With 50% maintenance margin, liquidation occurs when account value < 10000
    let threshold = summary.liquidation_threshold(0.5).unwrap();
    assert_eq!(threshold, 10000.0); // 5000 / 0.5 = 10000

    // Test edge case: zero or negative maintenance margin ratio
    let result = summary.liquidation_threshold(0.0);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Maintenance margin ratio must be positive"));
}

#[tokio::test]
async fn test_liquidation_risk() {
    let summary = MarginSummary::new(
        "10000.0".to_string(),
        "2000.0".to_string(),
        "5000.0".to_string(),
        "8000.0".to_string(),
    );

    // Not at risk with 50% maintenance margin (threshold = 10000, current = 10000)
    let is_at_risk = summary.is_liquidation_risk(0.5).unwrap();
    assert!(!is_at_risk);

    // At risk if maintenance margin is higher
    let is_at_risk = summary.is_liquidation_risk(0.6).unwrap();
    assert!(is_at_risk); // Threshold = 8333.33, current = 10000, but with 60% margin it's riskier
}

#[tokio::test]
async fn test_available_margin() {
    let summary = MarginSummary::new(
        "10000.0".to_string(),
        "2000.0".to_string(),
        "5000.0".to_string(),
        "8000.0".to_string(),
    );

    let available_margin = summary.available_margin().unwrap();
    assert_eq!(available_margin, 8000.0); // 10000 - 2000 = 8000

    // Test with more margin used
    let summary_full = MarginSummary::new(
        "10000.0".to_string(),
        "10000.0".to_string(),
        "5000.0".to_string(),
        "8000.0".to_string(),
    );

    let available_margin = summary_full.available_margin().unwrap();
    assert_eq!(available_margin, 0.0); // 10000 - 10000 = 0
}

#[tokio::test]
async fn test_max_leverage() {
    let summary = MarginSummary::new(
        "10000.0".to_string(),
        "2000.0".to_string(),
        "5000.0".to_string(),
        "8000.0".to_string(),
    );

    let max_leverage = summary.max_leverage().unwrap();
    assert_eq!(max_leverage, 0.5); // 5000 / 10000 = 0.5

    // Test edge case: zero account value
    let summary_zero = MarginSummary::new(
        "0.0".to_string(),
        "2000.0".to_string(),
        "5000.0".to_string(),
        "8000.0".to_string(),
    );

    let result = summary_zero.max_leverage();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Account value cannot be zero"));
}

#[tokio::test]
async fn test_validate() {
    let summary = MarginSummary::new(
        "10000.0".to_string(),
        "2000.0".to_string(),
        "5000.0".to_string(),
        "8000.0".to_string(),
    );

    assert!(summary.validate().is_ok());

    // Test negative account value
    let summary_negative = MarginSummary::new(
        "-1000.0".to_string(),
        "2000.0".to_string(),
        "5000.0".to_string(),
        "8000.0".to_string(),
    );

    let result = summary_negative.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Account value cannot be negative"));

    // Test margin used exceeds account value
    let summary_over = MarginSummary::new(
        "1000.0".to_string(),
        "2000.0".to_string(),
        "5000.0".to_string(),
        "8000.0".to_string(),
    );

    let result = summary_over.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Total margin used cannot exceed account value"));

    // Test negative margin used
    let summary_neg_margin = MarginSummary::new(
        "10000.0".to_string(),
        "-500.0".to_string(),
        "5000.0".to_string(),
        "8000.0".to_string(),
    );

    let result = summary_neg_margin.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Total margin used cannot be negative"));
}

#[tokio::test]
async fn test_health_score() {
    let summary = MarginSummary::new(
        "10000.0".to_string(),
        "2000.0".to_string(),
        "5000.0".to_string(),
        "8000.0".to_string(),
    );

    let health_score = summary.health_score(0.5).unwrap();
    assert!(health_score > 0.0 && health_score <= 100.0);

    // Test with poor health (high utilization)
    let poor_summary = MarginSummary::new(
        "1000.0".to_string(),
        "900.0".to_string(),
        "5000.0".to_string(),
        "8000.0".to_string(),
    );

    let poor_health = poor_summary.health_score(0.5).unwrap();
    assert!(poor_health < health_score); // Should have worse health score

    // Test with excellent health (low utilization, lots of margin)
    let good_summary = MarginSummary::new(
        "100000.0".to_string(),
        "1000.0".to_string(),
        "5000.0".to_string(),
        "8000.0".to_string(),
    );

    let good_health = good_summary.health_score(0.5).unwrap();
    assert!(good_health > health_score); // Should have better health score
}

#[tokio::test]
async fn test_net_short_position() {
    // Test with negative total_ntl_pos (net short position)
    let short_summary = MarginSummary::new(
        "10000.0".to_string(),
        "2000.0".to_string(),
        "-5000.0".to_string(), // Net short position
        "8000.0".to_string(),
    );

    assert!(short_summary.validate().is_ok());

    // Maintenance margin calculation should work with negative positions
    let maintenance_margin = short_summary.maintenance_margin().unwrap();
    assert_eq!(maintenance_margin, -1.6); // 8000 / (-5000) = -1.6

    // Max leverage should also work
    let max_leverage = short_summary.max_leverage().unwrap();
    assert_eq!(max_leverage, -0.5); // (-5000) / 10000 = -0.5
}

#[tokio::test]
async fn test_edge_cases() {
    // Test zero values
    let zero_summary = MarginSummary::new(
        "0.0".to_string(),
        "0.0".to_string(),
        "0.0".to_string(),
        "0.0".to_string(),
    );

    // Should validate (zero account value is allowed, but will fail utilization calc)
    assert!(zero_summary.validate().is_ok());

    // Test very large values
    let large_summary = MarginSummary::new(
        "1000000000.0".to_string(),
        "500000000.0".to_string(),
        "2000000000.0".to_string(),
        "1500000000.0".to_string(),
    );

    assert!(large_summary.validate().is_ok());
    let utilization = large_summary.margin_utilization().unwrap();
    assert_eq!(utilization, 50.0);

    // Test decimal precision
    let precise_summary = MarginSummary::new(
        "12345.6789".to_string(),
        "2345.6789".to_string(),
        "5678.9012".to_string(),
        "9876.5432".to_string(),
    );

    assert!(precise_summary.validate().is_ok());
    let utilization = precise_summary.margin_utilization().unwrap();
    assert!((utilization - 19.0).abs() < 0.1); // Approximately 19%
}

#[tokio::test]
async fn test_serialization_roundtrip() {
    let original = MarginSummary::new(
        "10000.0".to_string(),
        "2000.0".to_string(),
        "5000.0".to_string(),
        "8000.0".to_string(),
    );

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: MarginSummary = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.account_value(), "10000.0");
    assert_eq!(deserialized.total_margin_used(), "2000.0");
    assert_eq!(deserialized.total_ntl_pos(), "5000.0");
    assert_eq!(deserialized.total_raw_usd(), "8000.0");

    // Test that calculations work on deserialized struct
    assert!(deserialized.validate().is_ok());
    assert_eq!(deserialized.margin_utilization().unwrap(), 20.0);
}