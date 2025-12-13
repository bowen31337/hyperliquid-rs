//! Common types used throughout the Hyperliquid SDK

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod address;
pub use address::Address;

pub mod precision;
pub use precision::{
    OrderWireBuilder, PegPriceType, PrecisionError, TriggerCondition,
    float_to_int, float_to_int_for_hashing, float_to_usd_int, float_to_wire,
    validate_price_precision, validate_quantity_precision, validate_usd_precision
};

pub mod timestamp;
pub use timestamp::{
    add_millis, add_seconds, format_timestamp, get_timestamp_ms, get_timestamp_seconds,
    is_valid_timestamp, millis_to_seconds, seconds_to_millis, time_diff_ms, time_diff_seconds,
    TimestampError, validate_future_timestamp, validate_past_timestamp
};

pub mod perp_schema;
pub use perp_schema::{
    PerpDexSchemaInput, PerpDexSchemaInputBuilder, PerpDexSchemaInputWithAddress
};

pub mod optimized;
pub use optimized::{SymbolInterner, SymbolId, OptimizedOrder, OrderSide, OrderType, OptimizedPosition, OptimizedL2Book, OptimizedTrade, OptimizedUserState, TradingObjectPool, TradingAllocator, TradingAllocatorStats};

pub mod response_utils;
pub use response_utils::{ApiResponse, parse_response, parse_success_response, parse_error_response, wrap_success, wrap_error, is_error_response, extract_status, extract_nested_data};

/// Account role and permission types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    /// Account type (e.g., "user", "admin", "subaccount")
    pub account_type: String,
    /// List of permissions for this account
    pub permissions: Vec<String>,
    /// Account status (active, suspended, etc.)
    pub status: String,
    /// Timestamp when role was assigned
    pub assigned_at: i64,
}

/// Account role response from userRole endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRoleResponse {
    /// User address
    pub user: Address,
    /// Account role information
    pub role: UserRole,
}

/// TWAP execution types
pub use twap::{TwapSliceFill, TwapSlice, TwapExecutionSummary, ExecutionQuality, TwapSliceFillsResponse};

pub mod twap;

/// Staking summary for a user including total delegated and rewards
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StakingSummary {
    /// Total amount delegated across all validators
    pub total_delegated: String,
    /// Total pending rewards across all delegations
    pub total_pending_rewards: String,
    /// Number of active delegations
    pub delegation_count: i32,
    /// Total earned rewards (claimed + pending)
    pub total_earned_rewards: String,
}

/// Individual delegation to a validator
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Delegation {
    /// Validator address
    pub validator_address: String,
    /// Amount delegated to this validator
    pub amount: String,
    /// Pending rewards from this delegation
    pub pending_rewards: String,
    /// Status of delegation (active, pending, etc.)
    pub status: String,
    /// Delegation timestamp (Unix milliseconds)
    pub delegated_at: i64,
    /// Last reward claim timestamp (Unix milliseconds)
    pub last_claimed_at: Option<i64>,
}

/// Staking rewards history
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StakingRewards {
    /// Total claimed rewards
    pub total_claimed: String,
    /// Total pending rewards
    pub total_pending: String,
    /// Reward history entries
    pub history: Vec<RewardEvent>,
}

/// Individual reward event (claim or accrual)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardEvent {
    /// Type of reward event
    pub event_type: RewardEventType,
    /// Validator address
    pub validator_address: String,
    /// Amount of rewards
    pub amount: String,
    /// Timestamp (Unix milliseconds)
    pub timestamp: i64,
    /// Transaction hash if applicable
    pub tx_hash: Option<String>,
}

/// Type of reward event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RewardEventType {
    /// Rewards accrued (earned)
    Accrued,
    /// Rewards claimed by user
    Claimed,
    /// Delegation rewards for new delegation
    Delegated,
    /// Undelegation with pending rewards
    Undelegated,
}

/// Comprehensive delegator history including all events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegatorHistory {
    /// All delegation-related events
    pub events: Vec<DelegatorEvent>,
    /// Summary statistics
    pub summary: DelegatorSummary,
}

/// Individual delegator event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegatorEvent {
    /// Type of event
    pub event_type: DelegatorEventType,
    /// Validator address involved
    pub validator_address: String,
    /// Amount involved in the event
    pub amount: String,
    /// Timestamp (Unix milliseconds)
    pub timestamp: i64,
    /// Transaction hash if applicable
    pub tx_hash: Option<String>,
    /// Status of the event
    pub status: String,
}

/// Type of delegator event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DelegatorEventType {
    /// Delegation created
    Delegated,
    /// Delegation increased
    DelegatedMore,
    /// Delegation decreased
    Undelegated,
    /// Full undelegation
    UndelegatedAll,
    /// Rewards claimed
    RewardsClaimed,
    /// Validator slashed
    Slashed,
}

/// Delegator summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegatorSummary {
    /// Total lifetime delegated
    pub total_delegated_lifetime: String,
    /// Total lifetime rewards
    pub total_rewards_lifetime: String,
    /// Total lifetime slashed
    pub total_slashed_lifetime: String,
    /// Current active delegations
    pub current_delegation_count: i32,
    /// First delegation timestamp
    pub first_delegation_at: Option<i64>,
    /// Last activity timestamp
    pub last_activity_at: i64,
}

/// Validator information for staking
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorInfo {
    /// Validator address
    pub address: String,
    /// Validator name
    pub name: String,
    /// Commission rate (percentage)
    pub commission_rate: String,
    /// Total staked amount
    pub total_staked: String,
    /// Number of delegators
    pub delegator_count: i32,
    /// Validator status (active, jailed, etc.)
    pub status: String,
    /// Description
    pub description: Option<String>,
    /// Website URL
    pub website: Option<String>,
    /// Creation timestamp
    pub created_at: i64,
}

/// Staking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StakingConfig {
    /// Minimum delegation amount
    pub min_delegation: String,
    /// Maximum number of validators per user
    pub max_validators_per_user: i32,
    /// Unbonding period in milliseconds
    pub unbonding_period_ms: i64,
    /// Reward distribution interval
    pub reward_distribution_interval: i64,
}

/// Environment configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Environment {
    Mainnet,
    Testnet,
    Local,
}

/// API endpoint URLs for different environments
impl Environment {
    pub fn base_url(&self) -> &'static str {
        match self {
            Environment::Mainnet => "https://api.hyperliquid.xyz",
            Environment::Testnet => "https://api.hyperliquid-testnet.xyz",
            Environment::Local => "http://localhost:3001",
        }
    }

    pub fn websocket_url(&self) -> &'static str {
        match self {
            Environment::Mainnet => "wss://api.hyperliquid.xyz/ws",
            Environment::Testnet => "wss://api.hyperliquid-testnet.xyz/ws",
            Environment::Local => "ws://localhost:3001/ws",
        }
    }
}

/// Market type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarketType {
    #[serde(rename = "coin-margined_futures")]
    CoinMarginedFutures,
    #[serde(rename = "coin-margined_options")]
    CoinMarginedOptions,
    #[serde(rename = "usd-margined_futures")]
    UsdMarginedFutures,
    #[serde(rename = "usd-margined_options")]
    UsdMarginedOptions,
    #[serde(rename = "spot")]
    Spot,
}

/// Subscription types for WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Subscription {
    #[serde(rename = "allMids")]
    AllMids,
    #[serde(rename = "l2Book")]
    L2Book { coin: String },
    #[serde(rename = "trades")]
    Trades { coin: String },
    #[serde(rename = "bbo")]
    Bbo { coin: String },
    #[serde(rename = "candle")]
    Candle { coin: String, interval: String },
    #[serde(rename = "userEvents")]
    UserEvents { user: Address },
    #[serde(rename = "userFills")]
    UserFills { user: Address },
    #[serde(rename = "orderUpdates")]
    OrderUpdates { user: Address },
    #[serde(rename = "userFundings")]
    UserFundings { user: Address },
    #[serde(rename = "userNonFundingLedgerUpdates")]
    UserNonFundingLedgerUpdates { user: Address },
    #[serde(rename = "webData2")]
    WebData2 { user: Address },
    #[serde(rename = "activeAssetCtx")]
    ActiveAssetCtx { coin: String },
    #[serde(rename = "activeAssetData")]
    ActiveAssetData { user: Address, coin: String },
}

/// Base response structure for API calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseResponse<T> {
    pub data: T,
}

/// Error response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: i32,
    pub msg: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

impl ErrorResponse {
    /// Create a new error response
    pub fn new(code: i32, msg: String, data: Option<serde_json::Value>) -> Self {
        Self { code, msg, data }
    }

    /// Convert to HyperliquidError
    pub fn to_error(&self) -> crate::error::HyperliquidError {
        crate::error::HyperliquidError::Client {
            code: self.code,
            message: self.msg.clone(),
            data: self.data.clone(),
        }
    }
}

/// Result type that can be either a success or error response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Success(BaseResponse<T>),
    Error(ErrorResponse),
}

impl<T> ApiResponse<T> {
    /// Check if this is a success response
    pub fn is_success(&self) -> bool {
        matches!(self, ApiResponse::Success(_))
    }

    /// Check if this is an error response
    pub fn is_error(&self) -> bool {
        matches!(self, ApiResponse::Error(_))
    }

    /// Extract the data from success response, or return error
    pub fn into_result(self) -> Result<T, ErrorResponse> {
        match self {
            ApiResponse::Success(base) => Ok(base.data),
            ApiResponse::Error(err) => Err(err),
        }
    }

    /// Get the data from success response, or None if error
    pub fn data(self) -> Option<T> {
        match self {
            ApiResponse::Success(base) => Some(base.data),
            ApiResponse::Error(_) => None,
        }
    }

    /// Get the error from error response, or None if success
    pub fn error(self) -> Option<ErrorResponse> {
        match self {
            ApiResponse::Success(_) => None,
            ApiResponse::Error(err) => Some(err),
        }
    }
}

/// Utility functions for response parsing
pub mod response_utils {
    use super::{ApiResponse, BaseResponse, ErrorResponse};
    use serde::{de::DeserializeOwned, Serialize};
    use serde_json::Value;

    /// Parse a response string into an ApiResponse
    pub fn parse_response<T>(text: &str) -> Result<ApiResponse<T>, serde_json::Error>
    where
        T: DeserializeOwned,
    {
        // Try to parse as ApiResponse first (which handles both success and error)
        let api_response: ApiResponse<T> = serde_json::from_str(text)?;
        Ok(api_response)
    }

    /// Parse a success response directly to the data type
    pub fn parse_success_response<T>(text: &str) -> Result<T, ErrorResponse>
    where
        T: DeserializeOwned,
    {
        let api_response: ApiResponse<T> = serde_json::from_str(text)
            .map_err(|_| ErrorResponse::new(-1, "Failed to parse response".to_string(), None))?;

        match api_response {
            ApiResponse::Success(base) => Ok(base.data),
            ApiResponse::Error(err) => Err(err),
        }
    }

    /// Parse an error response
    pub fn parse_error_response(text: &str) -> Result<ErrorResponse, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Create a success response wrapper
    pub fn wrap_success<T>(data: T) -> BaseResponse<T> {
        BaseResponse { data }
    }

    /// Create an error response wrapper
    pub fn wrap_error(code: i32, msg: String, data: Option<Value>) -> ErrorResponse {
        ErrorResponse { code, msg, data }
    }

    /// Check if a response string represents an error
    pub fn is_error_response(text: &str) -> bool {
        // Try to parse as ErrorResponse first
        if let Ok(_err) = serde_json::from_str::<ErrorResponse>(text) {
            return true;
        }

        // Check if it has error fields
        if let Ok(value) = serde_json::from_str::<Value>(text) {
            if value.get("code").is_some() && value.get("msg").is_some() {
                return true;
            }
        }

        false
    }

    /// Extract status field from a response if present
    pub fn extract_status(text: &str) -> Option<String> {
        if let Ok(value) = serde_json::from_str::<Value>(text) {
            value.get("status").and_then(|v| v.as_str()).map(|s| s.to_string())
        } else {
            None
        }
    }

    /// Extract data field from a nested response
    pub fn extract_nested_data<T>(text: &str) -> Result<Option<T>, serde_json::Error>
    where
        T: DeserializeOwned,
    {
        let value: Value = serde_json::from_str(text)?;

        // Try direct data field
        if let Some(data) = value.get("data") {
            return Ok(Some(serde_json::from_value(data.clone())?));
        }

        // Try nested data structures
        if let Some(result) = value.get("result") {
            if let Some(data) = result.get("data") {
                return Ok(Some(serde_json::from_value(data.clone())?));
            }
        }

        Ok(None)
    }
}

/// Meta information about assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub universe: Vec<AssetMeta>,
    pub exchange: Option<ExchangeMeta>,
}

/// Asset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMeta {
    pub name: String,
    pub onlyIsolated: bool,
    pub szDecimals: i32,
    pub maxLeverage: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxDynamicLeverage: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<Vec<AssetMeta>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxOi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underlying: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isInverse: Option<bool>,
}

/// Exchange metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeMeta {
    pub vaults: Vec<VaultMeta>,
}

/// Vault metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultMeta {
    pub vault: Address,
    pub name: String,
    pub creator: Address,
    pub creatorLong: String,
    pub creatorShort: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
}

/// User state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserState {
    pub marginSummary: MarginSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crossMarginSummary: Option<CrossMarginSummary>,
    pub positions: Vec<Position>,
    pub withdrawable: String,
    pub assetPositions: Vec<AssetPosition>,
}

/// Margin summary with calculation methods
///
/// This struct represents the margin summary for a trading account,
/// including account value, margin usage, and position values.
///
/// Calculations:
/// - Account Value = Total Raw USD + Total Unrealized PnL
/// - Total Margin Used = Sum of margin used by all positions
/// - Total NTL POS = Net Total Long/Short Position Value
/// - Total Raw USD = Sum of raw position values
///
/// # Examples
/// ```
/// use hyperliquid_core::types::MarginSummary;
///
/// let summary = MarginSummary::new("10000.0", "2000.0", "5000.0", "8000.0");
/// assert_eq!(summary.account_value(), "10000.0");
/// assert_eq!(summary.total_margin_used(), "2000.0");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginSummary {
    pub accountValue: String,
    pub totalMarginUsed: String,
    pub totalNtlPos: String,
    pub totalRawUsd: String,
}

impl MarginSummary {
    /// Create a new MarginSummary with the given values
    ///
    /// # Arguments
    /// * `account_value` - Total account value in USD
    /// * `total_margin_used` - Total margin currently used
    /// * `total_ntl_pos` - Total net position value
    /// * `total_raw_usd` - Total raw USD value of positions
    ///
    /// # Returns
    /// A new MarginSummary instance
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::MarginSummary;
    ///
    /// let summary = MarginSummary::new(
    ///     "10000.0".to_string(),
    ///     "2000.0".to_string(),
    ///     "5000.0".to_string(),
    ///     "8000.0".to_string(),
    /// );
    /// ```
    pub fn new(
        account_value: String,
        total_margin_used: String,
        total_ntl_pos: String,
        total_raw_usd: String,
    ) -> Self {
        Self {
            accountValue: account_value,
            totalMarginUsed: total_margin_used,
            totalNtlPos: total_ntl_pos,
            totalRawUsd: total_raw_usd,
        }
    }

    /// Get the account value
    pub fn account_value(&self) -> &str {
        &self.accountValue
    }

    /// Get the total margin used
    pub fn total_margin_used(&self) -> &str {
        &self.totalMarginUsed
    }

    /// Get the total net position value
    pub fn total_ntl_pos(&self) -> &str {
        &self.totalNtlPos
    }

    /// Get the total raw USD value
    pub fn total_raw_usd(&self) -> &str {
        &self.totalRawUsd
    }

    /// Calculate the margin utilization ratio
    ///
    /// Returns the ratio of margin used to account value as a percentage
    ///
    /// # Returns
    /// * `Result<f64, String>` - Margin utilization percentage or error
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::MarginSummary;
    ///
    /// let summary = MarginSummary::new(
    ///     "10000.0".to_string(),
    ///     "2000.0".to_string(),
    ///     "5000.0".to_string(),
    ///     "8000.0".to_string(),
    /// );
    ///
    /// let utilization = summary.margin_utilization().unwrap();
    /// assert_eq!(utilization, 20.0); // 20% utilization
    /// ```
    pub fn margin_utilization(&self) -> Result<f64, String> {
        let account_value = self.account_value.parse::<f64>()
            .map_err(|e| format!("Failed to parse accountValue as f64: {}", e))?;
        let margin_used = self.total_margin_used.parse::<f64>()
            .map_err(|e| format!("Failed to parse totalMarginUsed as f64: {}", e))?;

        if account_value == 0.0 {
            return Err("Account value cannot be zero".to_string());
        }

        Ok((margin_used / account_value) * 100.0)
    }

    /// Calculate the maintenance margin ratio
    ///
    /// This is the ratio of total raw USD to total net position value
    ///
    /// # Returns
    /// * `Result<f64, String>` - Maintenance margin ratio or error
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::MarginSummary;
    ///
    /// let summary = MarginSummary::new(
    ///     "10000.0".to_string(),
    ///     "2000.0".to_string(),
    ///     "5000.0".to_string(),
    ///     "8000.0".to_string(),
    /// );
    ///
    /// let maintenance_margin = summary.maintenance_margin().unwrap();
    /// assert_eq!(maintenance_margin, 1.6); // 160% maintenance margin
    /// ```
    pub fn maintenance_margin(&self) -> Result<f64, String> {
        let total_raw_usd = self.total_raw_usd.parse::<f64>()
            .map_err(|e| format!("Failed to parse totalRawUsd as f64: {}", e))?;
        let total_ntl_pos = self.total_ntl_pos.parse::<f64>()
            .map_err(|e| format!("Failed to parse totalNtlPos as f64: {}", e))?;

        if total_ntl_pos == 0.0 {
            return Err("Total NTL position cannot be zero".to_string());
        }

        Ok(total_raw_usd / total_ntl_pos)
    }

    /// Calculate the liquidation threshold
    ///
    /// This returns the account value at which liquidation would occur
    /// based on the maintenance margin requirements
    ///
    /// # Arguments
    /// * `maintenance_margin_ratio` - Required maintenance margin ratio (e.g., 0.5 for 50%)
    ///
    /// # Returns
    /// * `Result<f64, String>` - Liquidation threshold or error
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::MarginSummary;
    ///
    /// let summary = MarginSummary::new(
    ///     "10000.0".to_string(),
    ///     "2000.0".to_string(),
    ///     "5000.0".to_string(),
    ///     "8000.0".to_string(),
    /// );
    ///
    /// let liquidation_threshold = summary.liquidation_threshold(0.5).unwrap();
    /// // Would liquidate if account value drops below 2500
    /// ```
    pub fn liquidation_threshold(&self, maintenance_margin_ratio: f64) -> Result<f64, String> {
        let total_ntl_pos = self.total_ntl_pos.parse::<f64>()
            .map_err(|e| format!("Failed to parse totalNtlPos as f64: {}", e))?;

        // Liquidation occurs when account value * maintenance_margin_ratio < total_ntl_pos
        // Therefore liquidation threshold = total_ntl_pos / maintenance_margin_ratio
        if maintenance_margin_ratio <= 0.0 {
            return Err("Maintenance margin ratio must be positive".to_string());
        }

        Ok(total_ntl_pos / maintenance_margin_ratio)
    }

    /// Check if the account is at risk of liquidation
    ///
    /// # Arguments
    /// * `maintenance_margin_ratio` - Required maintenance margin ratio
    ///
    /// # Returns
    /// * `Result<bool, String>` - True if at risk of liquidation
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::MarginSummary;
    ///
    /// let summary = MarginSummary::new(
    ///     "10000.0".to_string(),
    ///     "2000.0".to_string(),
    ///     "5000.0".to_string(),
    ///     "8000.0".to_string(),
    /// );
    ///
    /// let is_at_risk = summary.is_liquidation_risk(0.5).unwrap();
    /// assert!(!is_at_risk); // Not at risk with 50% maintenance margin
    /// ```
    pub fn is_liquidation_risk(&self, maintenance_margin_ratio: f64) -> Result<bool, String> {
        let account_value = self.account_value.parse::<f64>()
            .map_err(|e| format!("Failed to parse accountValue as f64: {}", e))?;
        let liquidation_threshold = self.liquidation_threshold(maintenance_margin_ratio)?;

        Ok(account_value < liquidation_threshold)
    }

    /// Calculate the amount of margin available for new positions
    ///
    /// This is the difference between account value and current margin used
    ///
    /// # Returns
    /// * `Result<f64, String>` - Available margin in USD
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::MarginSummary;
    ///
    /// let summary = MarginSummary::new(
    ///     "10000.0".to_string(),
    ///     "2000.0".to_string(),
    ///     "5000.0".to_string(),
    ///     "8000.0".to_string(),
    /// );
    ///
    /// let available_margin = summary.available_margin().unwrap();
    /// assert_eq!(available_margin, 8000.0); // 8000 USD available
    /// ```
    pub fn available_margin(&self) -> Result<f64, String> {
        let account_value = self.account_value.parse::<f64>()
            .map_err(|e| format!("Failed to parse accountValue as f64: {}", e))?;
        let total_margin_used = self.total_margin_used.parse::<f64>()
            .map_err(|e| format!("Failed to parse totalMarginUsed as f64: {}", e))?;

        Ok(account_value - total_margin_used)
    }

    /// Calculate the maximum leverage available
    ///
    /// This is the ratio of total position value to account value
    ///
    /// # Returns
    /// * `Result<f64, String>` - Maximum leverage ratio
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::MarginSummary;
    ///
    /// let summary = MarginSummary::new(
    ///     "10000.0".to_string(),
    ///     "2000.0".to_string(),
    ///     "5000.0".to_string(),
    ///     "8000.0".to_string(),
    /// );
    ///
    /// let max_leverage = summary.max_leverage().unwrap();
    /// assert_eq!(max_leverage, 0.5); // 0.5x leverage (5000/10000)
    /// ```
    pub fn max_leverage(&self) -> Result<f64, String> {
        let account_value = self.account_value.parse::<f64>()
            .map_err(|e| format!("Failed to parse accountValue as f64: {}", e))?;
        let total_ntl_pos = self.total_ntl_pos.parse::<f64>()
            .map_err(|e| format!("Failed to parse totalNtlPos as f64: {}", e))?;

        if account_value == 0.0 {
            return Err("Account value cannot be zero".to_string());
        }

        Ok(total_ntl_pos / account_value)
    }

    /// Validate the margin summary consistency
    ///
    /// Checks that the values are internally consistent
    ///
    /// # Returns
    /// * `Result<(), String>` - Ok if valid, error message if invalid
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::MarginSummary;
    ///
    /// let summary = MarginSummary::new(
    ///     "10000.0".to_string(),
    ///     "2000.0".to_string(),
    ///     "5000.0".to_string(),
    ///     "8000.0".to_string(),
    /// );
    ///
    /// assert!(summary.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<(), String> {
        // Parse all values
        let account_value = self.account_value.parse::<f64>()
            .map_err(|e| format!("Invalid accountValue: {}", e))?;
        let total_margin_used = self.total_margin_used.parse::<f64>()
            .map_err(|e| format!("Invalid totalMarginUsed: {}", e))?;
        let total_ntl_pos = self.total_ntl_pos.parse::<f64>()
            .map_err(|e| format!("Invalid totalNtlPos: {}", e))?;
        let total_raw_usd = self.total_raw_usd.parse::<f64>()
            .map_err(|e| format!("Invalid totalRawUsd: {}", e))?;

        // Validate ranges
        if account_value < 0.0 {
            return Err("Account value cannot be negative".to_string());
        }
        if total_margin_used < 0.0 {
            return Err("Total margin used cannot be negative".to_string());
        }
        if total_raw_usd < 0.0 {
            return Err("Total raw USD cannot be negative".to_string());
        }

        // Validate relationships
        if total_margin_used > account_value {
            return Err("Total margin used cannot exceed account value".to_string());
        }

        // Note: total_ntl_pos can be negative (net short position)

        Ok(())
    }

    /// Calculate the account health score (0-100)
    ///
    /// Higher scores indicate better account health
    ///
    /// # Arguments
    /// * `maintenance_margin_ratio` - Required maintenance margin ratio
    ///
    /// # Returns
    /// * `Result<f64, String>` - Health score from 0 (poor) to 100 (excellent)
    ///
    /// # Examples
    /// ```
    /// use hyperliquid_core::types::MarginSummary;
    ///
    /// let summary = MarginSummary::new(
    ///     "10000.0".to_string(),
    ///     "2000.0".to_string(),
    ///     "5000.0".to_string(),
    ///     "8000.0".to_string(),
    /// );
    ///
    /// let health_score = summary.health_score(0.5).unwrap();
    /// assert!(health_score > 50.0); // Good health
    /// ```
    pub fn health_score(&self, maintenance_margin_ratio: f64) -> Result<f64, String> {
        // Validate the summary
        self.validate()?;

        // Calculate margin utilization (0-100)
        let utilization = self.margin_utilization()?;
        let utilization_score = (100.0 - utilization).max(0.0).min(100.0);

        // Calculate distance to liquidation
        let account_value = self.account_value.parse::<f64>()?;
        let liquidation_threshold = self.liquidation_threshold(maintenance_margin_ratio)?;
        let distance_to_liquidation = if account_value >= liquidation_threshold {
            (account_value - liquidation_threshold) / account_value * 100.0
        } else {
            0.0
        };

        // Calculate available margin ratio
        let available_margin = self.available_margin()?;
        let available_margin_score = (available_margin / account_value * 100.0).max(0.0).min(100.0);

        // Weighted average: 50% utilization, 30% distance to liquidation, 20% available margin
        let health_score = utilization_score * 0.5 + distance_to_liquidation * 0.3 + available_margin_score * 0.2;

        Ok(health_score.max(0.0).min(100.0))
    }
}

/// Cross margin summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossMarginSummary {
    pub accountValue: String,
    pub totalMarginUsed: String,
    pub totalNtlPos: String,
    pub totalRawUsd: String,
}

/// Position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub coin: String,
    pub position: PositionDetails,
}

/// Detailed position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionDetails {
    pub szi: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entryPx: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leverage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liquidationPx: Option<String>,
    pub positionValue: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marginUsed: Option<String>,
    pub openSize: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rawPNL: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub returnOnEquity: Option<String>,
    pub type_: String,
    pub userID: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cumFunding: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxCost: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxLeverage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub positionUUID: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pendingFunding: Option<String>,
}

/// Asset position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPosition {
    pub time: i64,
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deltaUsd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub totalUsd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
}

/// Builder information for builder fees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderInfo {
    /// Builder wallet address
    pub b: String,
    /// Fee rate in tenths of basis points
    /// For example, 50 = 0.05% = 5 bps
    #[serde(rename = "f")]
    pub fee_in_tenths_bps: u32,
}

/// L2 order book snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2BookSnapshot {
    pub coin: String,
    pub levels: [Vec<OrderLevel>; 2],
    pub time: i64,
}

/// Order book level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderLevel {
    pub px: String,
    pub sz: String,
    pub n: i64,
    pub numLevels: Option<i64>,
}

/// Trade information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub coin: String,
    pub side: String,
    pub px: String,
    pub sz: String,
    pub time: i64,
    pub hash: Option<String>,
}

/// BBO (Best Bid/Offer) information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bbo {
    pub coin: String,
    pub bid: Option<BboLevel>,
    pub ask: Option<BboLevel>,
    pub time: i64,
}

/// BBO level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BboLevel {
    pub px: String,
    pub sz: String,
    pub mm: Option<String>,
}

/// Candle information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub coin: String,
    pub interval: String,
    pub start: i64,
    pub end: i64,
    pub trades: Option<i64>,
    pub txHash: Option<String>,
    pub open: String,
    pub close: String,
    pub high: String,
    pub low: String,
    pub volume: String,
    pub vwap: String,
    pub bidVolume: Option<String>,
    pub bidVwap: Option<String>,
    pub askVolume: Option<String>,
    pub askVwap: Option<String>,
}

/// Mid price information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidPrice {
    pub coin: String,
    pub mid: String,
    pub time: i64,
}

/// User event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEvent {
    pub cleared: Option<Cleared>,
    pub closedPnls: Option<Vec<ClosedPnl>>,
    pub deposits: Option<Vec<Deposit>>,
    pub fundingPayments: Option<Vec<FundingPayment>>,
    pub liquidations: Option<Vec<Liquidation>>,
    pub newOrders: Option<Vec<NewOrder>>,
    pub orderStatus: Option<Vec<OrderStatus>>,
    pub positions: Option<Vec<PositionUpdate>>,
    pub time: i64,
    pub tokenTransfers: Option<Vec<TokenTransfer>>,
    pub withFees: Option<Vec<WithFee>>,
    pub totalAbsFees: Option<String>,
    pub withFunds: Option<Vec<WithFund>>,
    pub pnlAnnouncements: Option<Vec<PnlAnnouncement>>,
    pub withdrawals: Option<Vec<Withdrawal>>,
    pub spotFills: Option<Vec<SpotFill>>,
    pub spotUserEvent: Option<SpotUserEvent>,
    pub crossMarginAccount: Option<CrossMarginAccount>,
    pub crossMarginAccountTransfer: Option<Vec<CrossMarginAccountTransfer>>,
    pub crossMarginStatus: Option<CrossMarginStatus>,
    pub crossMarginTx: Option<CrossMarginTx>,
}

/// Various event types - simplified for now
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cleared {
    pub accountValue: String,
    pub crossMarginSummary: Option<CrossMarginSummary>,
    pub type_: String,
    pub vault: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosedPnl {
    pub closedPnl: String,
    pub coin: String,
    pub dir: String,
    pub oid: i64,
    pub closedPnlPct: Option<String>,
    pub fee: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deposit {
    pub amount: String,
    pub coin: String,
    pub hash: String,
    pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingPayment {
    pub coin: String,
    pub fundingPayment: String,
    pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Liquidation {
    pub liquidation: LiquidationDetails,
    pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidationDetails {
    pub coin: String,
    pub dir: String,
    pub closedPnl: String,
    pub crossMarginSummary: Option<CrossMarginSummary>,
    pub marginSummary: MarginSummary,
    pub positions: Vec<Position>,
    pub vault: Option<String>,
    pub withdrawable: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewOrder {
    pub coin: String,
    pub coinOrderOpt: Option<String>,
    pub isPositionTpsl: Option<bool>,
    pub isTrigger: Option<bool>,
    pub limitPx: String,
    pub oid: i64,
    pub orderType: OrderType,
    pub pegOffsetValue: Option<String>,
    pub pegPriceType: Option<PegPriceType>,
    pub sz: String,
    pub time: i64,
    pub reduceOnly: Option<bool>,
    pub cloid: Option<String>,
    pub triggerCondition: Option<TriggerCondition>,
    pub triggerPx: Option<String>,
    pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatus {
    pub coin: String,
    pub oid: Option<i64>,
    pub status: String,
    pub type_: String,
    pub cloid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionUpdate {
    pub coin: String,
    pub position: PositionDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransfer {
    pub coin: String,
    pub dir: String,
    pub from: String,
    pub to: String,
    pub type_: String,
    pub usd: String,
    pub vault: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithFee {
    pub coin: String,
    pub fee: String,
    pub orderType: OrderType,
    pub oid: i64,
    pub px: String,
    pub sz: String,
    pub time: i64,
    pub type_: String,
    pub dir: Option<String>,
    pub cloid: Option<String>,
    pub triggerCondition: Option<TriggerCondition>,
    pub triggerPx: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithFund {
    pub amount: String,
    pub coin: String,
    pub dir: String,
    pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnlAnnouncement {
    pub amount: String,
    pub coin: String,
    pub type_: String,
    pub vault: Option<String>,
    pub vaultPnls: Option<Vec<VaultPnl>>,
    pub vaultPnlSum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultPnl {
    pub vault: String,
    pub vaultPnl: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Withdrawal {
    pub amount: String,
    pub coin: String,
    pub hash: String,
    pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotFill {
    pub coin: String,
    pub dir: String,
    pub orderType: OrderType,
    pub px: String,
    pub sz: String,
    pub time: i64,
    pub type_: String,
    pub oid: Option<i64>,
    pub cloid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotUserEvent {
    pub deposits: Option<Vec<Deposit>>,
    pub liquidations: Option<Vec<SpotLiquidation>>,
    pub newSpotOrders: Option<Vec<NewSpotOrder>>,
    pub orderStatus: Option<Vec<SpotOrderStatus>>,
    pub tokenTransfers: Option<Vec<SpotTokenTransfer>>,
    pub withdrawable: String,
    pub withFees: Option<Vec<SpotWithFee>>,
    pub withFunds: Option<Vec<SpotWithFund>>,
    pub withdrawals: Option<Vec<Withdrawal>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotLiquidation {
    pub liquidation: SpotLiquidationDetails,
    pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotLiquidationDetails {
    pub coin: String,
    pub dir: String,
    pub closedPnl: String,
    pub positions: Vec<SpotPosition>,
    pub withdrawable: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSpotOrder {
    pub coin: String,
    pub isPositionTpsl: Option<bool>,
    pub isTrigger: Option<bool>,
    pub limitPx: String,
    pub oid: i64,
    pub orderType: OrderType,
    pub pegOffsetValue: Option<String>,
    pub pegPriceType: Option<PegPriceType>,
    pub sz: String,
    pub time: i64,
    pub reduceOnly: Option<bool>,
    pub cloid: Option<String>,
    pub triggerCondition: Option<TriggerCondition>,
    pub triggerPx: Option<String>,
    pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotOrderStatus {
    pub coin: String,
    pub oid: Option<i64>,
    pub status: String,
    pub type_: String,
    pub cloid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotTokenTransfer {
    pub coin: String,
    pub dir: String,
    pub from: String,
    pub to: String,
    pub type_: String,
    pub usd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotWithFee {
    pub coin: String,
    pub fee: String,
    pub orderType: OrderType,
    pub oid: i64,
    pub px: String,
    pub sz: String,
    pub time: i64,
    pub type_: String,
    pub dir: Option<String>,
    pub cloid: Option<String>,
    pub triggerCondition: Option<TriggerCondition>,
    pub triggerPx: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotWithFund {
    pub amount: String,
    pub coin: String,
    pub dir: String,
    pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossMarginAccount {
    pub accountValue: String,
    pub crossMarginSummary: CrossMarginSummary,
    pub positions: Vec<Position>,
    pub withdrawable: String,
    pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossMarginAccountTransfer {
    pub amount: String,
    pub coin: String,
    pub dir: String,
    pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossMarginStatus {
    pub accountValue: String,
    pub crossMarginSummary: CrossMarginSummary,
    pub positions: Vec<Position>,
    pub status: CrossMarginStatusType,
    pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossMarginStatusType {
    #[serde(rename = "liquidate")]
    Liquidate,
    #[serde(rename = "unliquidate")]
    Unliquidate,
    #[serde(rename = "transfer")]
    Transfer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossMarginTx {
    pub amount: String,
    pub coin: String,
    pub dir: String,
    pub type_: String,
}

/// Order types for API compatibility
///
/// This enum represents the different order types supported by the Hyperliquid API.
/// The serde attributes ensure proper serialization/deserialization to match the API format.
///
/// Variants are renamed to match the API field names:
/// - `GoodTillCancel` serializes as "Gtc" for API compatibility
/// - Other variants use camelCase conversion
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OrderType {
    /// Limit order - executes at specified price or better
    Limit,
    /// Trigger order - executes when trigger conditions are met
    Trigger,
    /// Market order - executes immediately at best available price
    Market,
    /// Stop limit order - becomes limit order when stop price is reached
    StopLimit,
    /// Stop market order - becomes market order when stop price is reached
    StopMarket,
    /// Take profit limit order - becomes limit order when target price is reached
    TakeProfitLimit,
    /// Take profit market order - becomes market order when target price is reached
    TakeProfitMarket,
    /// Good Till Cancel - order remains active until cancelled
    #[serde(rename = "Gtc")]
    GoodTillCancel,
    /// Immediate Or Cancel - must be filled immediately, partial fills allowed
    #[serde(rename = "Ioc")]
    ImmediateOrCancel,
    /// Fill Or Kill - must be filled immediately and completely, or not at all
    #[serde(rename = "Fok")]
    FillOrKill,
    /// Auction Limit Order - used for auction phases
    #[serde(rename = "Alo")]
    AuctionLimitOrder,
}

/// Peg price types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PegPriceType {
    Mid,
    Oracle,
    Last,
    Opposite,
    OracleWithFallback,
}

/// Trigger conditions
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TriggerCondition {
    #[serde(rename = "mark")]
    Mark,
    #[serde(rename = "index")]
    Index,
    #[serde(rename = "last")]
    Last,
}

/// Spot asset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotAssetInfo {
    pub token: String,
    pub ctx: u32,
}

/// Spot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotMeta {
    pub name: String,
    pub onlyIsolated: bool,
    pub type_: Option<String>,
    pub tokens: Vec<SpotAssetInfo>,
}

/// Order wire format for API serialization
/// This is the format used when sending orders to the Hyperliquid API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderWire {
    /// Coin to trade
    pub coin: String,

    /// Client order ID (optional)
    pub cloid: Option<String>,

    /// Order ID (optional for cloid usage)
    #[serde(rename = "oid")]
    pub order_id: Option<i64>,

    /// Limit price
    #[serde(rename = "limitPx")]
    pub limit_price: String,

    /// Order size
    pub sz: String,

    /// Is buy order (true) or sell order (false)
    pub is_buy: bool,

    /// Reduce only flag
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub reduce_only: bool,

    /// Order type
    #[serde(rename = "orderType")]
    pub order_type: OrderType,

    /// Peg offset value for PEGGED orders
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peg_offset_value: Option<String>,

    /// Peg price type for PEGGED orders
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peg_price_type: Option<PegPriceType>,

    /// Is trigger order
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_trigger: Option<bool>,

    /// Trigger condition for TRIGGER orders
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_condition: Option<TriggerCondition>,

    /// Trigger price for TRIGGER orders
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_px: Option<String>,

    /// Timestamp (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,

    /// Type string (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,

    /// Coin order option (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coin_order_opt: Option<String>,

    /// Is position TP/SL (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_position_tpsl: Option<bool>,
}

/// WebSocket message type union (WsMsg)
/// Union type for different WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMsg {
    #[serde(rename = "allMids")]
    AllMidsMsg(AllMidsMsg),
    #[serde(rename = "l2Book")]
    L2BookMsg(L2BookMsg),
    #[serde(rename = "trades")]
    TradesMsg(TradesMsg),
    #[serde(rename = "bbo")]
    BboMsg(BboMsg),
    #[serde(rename = "candle")]
    CandleMsg(CandleMsg),
    #[serde(rename = "userEvents")]
    UserEventsMsg(UserEventsMsg),
    #[serde(rename = "userFills")]
    UserFillsMsg(UserFillsMsg),
    #[serde(rename = "orderUpdates")]
    OrderUpdatesMsg(OrderUpdatesMsg),
    #[serde(rename = "userFundings")]
    UserFundingsMsg(UserFundingsMsg),
    #[serde(rename = "pong")]
    PongMsg(PongMsg),
    #[serde(other)]
    OtherWsMsg(serde_json::Value),
}

/// AllMids WebSocket message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllMidsMsg {
    pub data: HashMap<String, String>,
    pub time: i64,
}

/// L2Book WebSocket message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2BookMsg {
    pub data: L2BookSnapshot,
}

/// Trades WebSocket message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradesMsg {
    pub data: Vec<Trade>,
}

/// BBO WebSocket message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BboMsg {
    pub data: Bbo,
}

/// Candle WebSocket message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandleMsg {
    pub data: Candle,
}

/// UserEvents WebSocket message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEventsMsg {
    pub data: UserEvent,
}

/// UserFills WebSocket message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFillsMsg {
    pub data: Vec<Fill>,
}

/// OrderUpdates WebSocket message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderUpdatesMsg {
    pub data: Vec<NewOrder>,
}

/// UserFundings WebSocket message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFundingsMsg {
    pub data: Vec<UserFunding>,
}

/// Pong WebSocket message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PongMsg {
    pub data: String, // "pong"
}

/// Fill trade record for perpetual trades
/// Represents a single trade execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
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
}

/// UserFunding record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFunding {
    /// Coin being traded
    pub coin: String,
    /// Funding payment amount (positive for received, negative for paid)
    pub funding: String,
    /// Timestamp of the funding payment
    pub time: i64,
}

impl Fill {
    /// Create a new Fill from trade data
    pub fn new(coin: String, side: String, px: String, sz: String, time: i64) -> Self {
        Self {
            coin,
            side,
            px,
            sz,
            time,
            hash: None,
            fee: None,
            fee_asset: None,
            oid: None,
        }
    }

    /// Get the fill price as f64
    pub fn price(&self) -> Result<f64, std::num::ParseFloatError> {
        self.px.parse()
    }

    /// Get the fill size as f64
    pub fn size(&self) -> Result<f64, std::num::ParseFloatError> {
        self.sz.parse()
    }

    /// Get the fee amount as f64 if available
    pub fn fee_amount(&self) -> Option<Result<f64, std::num::ParseFloatError>> {
        self.fee.as_ref().map(|f| f.parse())
    }
}

impl WsMsg {
    /// Check if this is a ping/pong message
    pub fn is_heartbeat(&self) -> bool {
        matches!(self, WsMsg::PongMsg(_))
    }

    /// Get the timestamp if available
    pub fn timestamp(&self) -> Option<i64> {
        match self {
            WsMsg::AllMidsMsg(msg) => Some(msg.time),
            WsMsg::L2BookMsg(_) => None, // L2BookMsg has time field but not at top level
            WsMsg::TradesMsg(_) => None,
            WsMsg::BboMsg(_) => None,
            WsMsg::CandleMsg(_) => None,
            WsMsg::UserEventsMsg(_) => None,
            WsMsg::UserFillsMsg(_) => None,
            WsMsg::OrderUpdatesMsg(_) => None,
            WsMsg::UserFundingsMsg(_) => None,
            WsMsg::PongMsg(_) => None,
            WsMsg::OtherWsMsg(_) => None,
        }
    }

    /// Get the channel/coin if applicable
    pub fn channel(&self) -> Option<String> {
        match self {
            WsMsg::AllMidsMsg(_) => Some("allMids".to_string()),
            WsMsg::L2BookMsg(msg) => Some(format!("l2Book.{}", msg.data.coin)),
            WsMsg::TradesMsg(msg) => {
                msg.data.first().map(|trade| format!("trades.{}", trade.coin))
            }
            WsMsg::BboMsg(msg) => Some(format!("bbo.{}", msg.data.coin)),
            WsMsg::CandleMsg(msg) => {
                Some(format!("candle.{}.{}", msg.data.coin, msg.data.interval))
            }
            WsMsg::UserEventsMsg(_) => Some("userEvents".to_string()),
            WsMsg::UserFillsMsg(_) => Some("userFills".to_string()),
            WsMsg::OrderUpdatesMsg(_) => Some("orderUpdates".to_string()),
            WsMsg::UserFundingsMsg(_) => Some("userFundings".to_string()),
            WsMsg::PongMsg(_) => Some("pong".to_string()),
            WsMsg::OtherWsMsg(_) => None,
        }
    }
}

impl OrderWire {
    /// Create a new limit order
    pub fn new_limit(
        coin: String,
        is_buy: bool,
        sz: String,
        limit_px: String,
    ) -> Self {
        Self {
            coin,
            cloid: None,
            order_id: None,
            limit_price: limit_px,
            sz,
            is_buy,
            reduce_only: false,
            order_type: OrderType::Limit,
            peg_offset_value: None,
            peg_price_type: None,
            is_trigger: None,
            trigger_condition: None,
            trigger_px: None,
            time: None,
            type_: None,
            coin_order_opt: None,
            is_position_tpsl: None,
        }
    }

    /// Create a new trigger order
    pub fn new_trigger(
        coin: String,
        is_buy: bool,
        sz: String,
        trigger_px: String,
        limit_px: String,
        condition: TriggerCondition,
    ) -> Self {
        Self {
            coin,
            cloid: None,
            order_id: None,
            limit_price: limit_px,
            sz,
            is_buy,
            reduce_only: false,
            order_type: OrderType::Trigger,
            peg_offset_value: None,
            peg_price_type: None,
            is_trigger: Some(true),
            trigger_condition: Some(condition),
            trigger_px: Some(trigger_px),
            time: None,
            type_: None,
            coin_order_opt: None,
            is_position_tpsl: None,
        }
    }

    /// Set client order ID
    pub fn with_cloid(mut self, cloid: String) -> Self {
        self.cloid = Some(cloid);
        self.order_id = None; // Use cloid instead of oid
        self
    }

    /// Set order ID
    pub fn with_oid(mut self, oid: i64) -> Self {
        self.order_id = Some(oid);
        self.cloid = None; // Use oid instead of cloid
        self
    }

    /// Set reduce only
    pub fn with_reduce_only(mut self, reduce_only: bool) -> Self {
        self.reduce_only = reduce_only;
        self
    }

    /// Set time
    pub fn with_time(mut self, time: i64) -> Self {
        self.time = Some(time);
        self
    }
}

/// Tests for WsMsg and Fill types
#[cfg(test)]
mod ws_msg_and_fill_tests {
    use super::*;

    #[test]
    fn test_ws_msg_all_mids_serialization() {
        // Test AllMidsMsg serialization/deserialization
        let mut data = HashMap::new();
        data.insert("BTC".to_string(), "50000.0".to_string());
        data.insert("ETH".to_string(), "3000.0".to_string());

        let all_mids_msg = AllMidsMsg {
            data: data.clone(),
            time: 1234567890,
        };

        let ws_msg = WsMsg::AllMidsMsg(all_mids_msg.clone());

        // Serialize to JSON
        let json = serde_json::to_string(&ws_msg).unwrap();
        println!("AllMidsMsg JSON: {}", json);

        // Deserialize back
        let deserialized: WsMsg = serde_json::from_str(&json).unwrap();

        match deserialized {
            WsMsg::AllMidsMsg(msg) => {
                assert_eq!(msg.data, data);
                assert_eq!(msg.time, 1234567890);
            }
            _ => panic!("Expected AllMidsMsg"),
        }

        // Test channel detection
        assert_eq!(ws_msg.channel(), Some("allMids".to_string()));
        assert_eq!(ws_msg.timestamp(), Some(1234567890));
    }

    #[test]
    fn test_ws_msg_l2_book_serialization() {
        // Test L2BookMsg serialization/deserialization
        let l2_book = L2BookSnapshot {
            coin: "BTC".to_string(),
            levels: [
                vec![OrderLevel {
                    px: "50000.0".to_string(),
                    sz: "1.0".to_string(),
                    n: 1,
                    numLevels: None,
                }],
                vec![OrderLevel {
                    px: "49999.0".to_string(),
                    sz: "0.5".to_string(),
                    n: 1,
                    numLevels: None,
                }],
            ],
            time: 1234567890,
        };

        let l2_book_msg = L2BookMsg { data: l2_book };
        let ws_msg = WsMsg::L2BookMsg(l2_book_msg);

        // Serialize to JSON
        let json = serde_json::to_string(&ws_msg).unwrap();
        println!("L2BookMsg JSON: {}", json);

        // Deserialize back
        let deserialized: WsMsg = serde_json::from_str(&json).unwrap();

        match deserialized {
            WsMsg::L2BookMsg(msg) => {
                assert_eq!(msg.data.coin, "BTC");
                assert_eq!(msg.data.levels[0].len(), 1);
                assert_eq!(msg.data.levels[1].len(), 1);
            }
            _ => panic!("Expected L2BookMsg"),
        }

        // Test channel detection
        assert_eq!(ws_msg.channel(), Some("l2Book.BTC".to_string()));
    }

    #[test]
    fn test_ws_msg_trades_serialization() {
        // Test TradesMsg serialization/deserialization
        let trades = vec![
            Trade {
                coin: "BTC".to_string(),
                side: "B".to_string(),
                px: "50000.0".to_string(),
                sz: "0.1".to_string(),
                time: 1234567890,
                hash: None,
            },
            Trade {
                coin: "BTC".to_string(),
                side: "S".to_string(),
                px: "49999.0".to_string(),
                sz: "0.05".to_string(),
                time: 1234567891,
                hash: None,
            },
        ];

        let trades_msg = TradesMsg { data: trades };
        let ws_msg = WsMsg::TradesMsg(trades_msg);

        // Serialize to JSON
        let json = serde_json::to_string(&ws_msg).unwrap();
        println!("TradesMsg JSON: {}", json);

        // Deserialize back
        let deserialized: WsMsg = serde_json::from_str(&json).unwrap();

        match deserialized {
            WsMsg::TradesMsg(msg) => {
                assert_eq!(msg.data.len(), 2);
                assert_eq!(msg.data[0].coin, "BTC");
                assert_eq!(msg.data[1].coin, "BTC");
            }
            _ => panic!("Expected TradesMsg"),
        }

        // Test channel detection
        assert_eq!(ws_msg.channel(), Some("trades.BTC".to_string()));
    }

    #[test]
    fn test_ws_msg_pong_serialization() {
        // Test PongMsg serialization/deserialization
        let pong_msg = PongMsg { data: "pong".to_string() };
        let ws_msg = WsMsg::PongMsg(pong_msg);

        // Serialize to JSON
        let json = serde_json::to_string(&ws_msg).unwrap();
        println!("PongMsg JSON: {}", json);

        // Deserialize back
        let deserialized: WsMsg = serde_json::from_str(&json).unwrap();

        match deserialized {
            WsMsg::PongMsg(msg) => {
                assert_eq!(msg.data, "pong");
            }
            _ => panic!("Expected PongMsg"),
        }

        // Test heartbeat detection
        assert!(ws_msg.is_heartbeat());
        assert_eq!(ws_msg.channel(), Some("pong".to_string()));
    }

    #[test]
    fn test_fill_creation_and_methods() {
        // Test Fill creation and utility methods
        let fill = Fill::new(
            "BTC".to_string(),
            "Buy".to_string(),
            "50000.0".to_string(),
            "0.1".to_string(),
            1234567890,
        );

        // Test basic fields
        assert_eq!(fill.coin, "BTC");
        assert_eq!(fill.side, "Buy");
        assert_eq!(fill.px, "50000.0");
        assert_eq!(fill.sz, "0.1");
        assert_eq!(fill.time, 1234567890);

        // Test utility methods
        assert_eq!(fill.price().unwrap(), 50000.0);
        assert_eq!(fill.size().unwrap(), 0.1);
        assert!(fill.fee_amount().is_none());

        // Test with fee
        let fill_with_fee = Fill {
            coin: "BTC".to_string(),
            side: "Buy".to_string(),
            px: "50000.0".to_string(),
            sz: "0.1".to_string(),
            time: 1234567890,
            hash: Some("0x123".to_string()),
            fee: Some("0.01".to_string()),
            fee_asset: Some("USDC".to_string()),
            oid: Some(12345),
        };

        assert_eq!(fill_with_fee.fee_amount().unwrap().unwrap(), 0.01);
        assert_eq!(fill_with_fee.hash, Some("0x123".to_string()));
        assert_eq!(fill_with_fee.oid, Some(12345));
    }

    #[test]
    fn test_fill_serialization() {
        // Test Fill serialization/deserialization
        let fill = Fill {
            coin: "ETH".to_string(),
            side: "Sell".to_string(),
            px: "3000.0".to_string(),
            sz: "2.0".to_string(),
            time: 1234567890,
            hash: Some("0x456".to_string()),
            fee: Some("0.02".to_string()),
            fee_asset: Some("ETH".to_string()),
            oid: Some(67890),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&fill).unwrap();
        println!("Fill JSON: {}", json);

        // Deserialize back
        let deserialized: Fill = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.coin, "ETH");
        assert_eq!(deserialized.side, "Sell");
        assert_eq!(deserialized.px, "3000.0");
        assert_eq!(deserialized.sz, "2.0");
        assert_eq!(deserialized.time, 1234567890);
        assert_eq!(deserialized.hash, Some("0x456".to_string()));
        assert_eq!(deserialized.fee, Some("0.02".to_string()));
        assert_eq!(deserialized.fee_asset, Some("ETH".to_string()));
        assert_eq!(deserialized.oid, Some(67890));
    }

    #[test]
    fn test_ws_msg_unknown_type() {
        // Test handling of unknown message types
        let unknown_json = r#"{"type": "unknownType", "data": {"custom": "value"}}"#;
        let ws_msg: WsMsg = serde_json::from_str(unknown_json).unwrap();

        match ws_msg {
            WsMsg::OtherWsMsg(data) => {
                let value: serde_json::Value = serde_json::from_str(r#"{"custom": "value"}"#).unwrap();
                assert_eq!(data, value);
            }
            _ => panic!("Expected OtherWsMsg"),
        }
    }

    #[test]
    fn test_user_fills_msg_serialization() {
        // Test UserFillsMsg with multiple fills
        let fills = vec![
            Fill::new("BTC".to_string(), "Buy".to_string(), "50000.0".to_string(), "0.1".to_string(), 1234567890),
            Fill::new("BTC".to_string(), "Sell".to_string(), "50100.0".to_string(), "0.05".to_string(), 1234567891),
        ];

        let user_fills_msg = UserFillsMsg { data: fills };
        let ws_msg = WsMsg::UserFillsMsg(user_fills_msg);

        // Serialize to JSON
        let json = serde_json::to_string(&ws_msg).unwrap();
        println!("UserFillsMsg JSON: {}", json);

        // Deserialize back
        let deserialized: WsMsg = serde_json::from_str(&json).unwrap();

        match deserialized {
            WsMsg::UserFillsMsg(msg) => {
                assert_eq!(msg.data.len(), 2);
                assert_eq!(msg.data[0].coin, "BTC");
                assert_eq!(msg.data[1].coin, "BTC");
            }
            _ => panic!("Expected UserFillsMsg"),
        }

        // Test channel detection
        assert_eq!(ws_msg.channel(), Some("userFills".to_string()));
    }

    #[test]
    fn test_error_handling_invalid_data() {
        // Test error handling for invalid JSON
        let invalid_json = r#"{"type": "allMids", "data": "invalid"}"#;
        let result: Result<WsMsg, _> = serde_json::from_str(invalid_json);

        // Should handle gracefully (depends on serde's strictness)
        // For now, just verify it doesn't panic
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod order_wire_tests {
    use super::*;

    #[test]
    fn test_limit_order_creation() {
        let order = OrderWire::new_limit(
            "BTC".to_string(),
            true,
            "0.1".to_string(),
            "50000".to_string(),
        );

        assert_eq!(order.coin, "BTC");
        assert_eq!(order.is_buy, true);
        assert_eq!(order.sz, "0.1");
        assert_eq!(order.limit_price, "50000");
        assert_eq!(order.order_type, OrderType::Limit);
        assert_eq!(order.reduce_only, false);
        assert!(order.cloid.is_none());
        assert!(order.order_id.is_none());
    }

    #[test]
    fn test_trigger_order_creation() {
        let order = OrderWire::new_trigger(
            "ETH".to_string(),
            false,
            "1.0".to_string(),
            "3000".to_string(),
            "2990".to_string(),
            TriggerCondition::Mark,
        );

        assert_eq!(order.coin, "ETH");
        assert_eq!(order.is_buy, false);
        assert_eq!(order.sz, "1.0");
        assert_eq!(order.limit_price, "2990");
        assert_eq!(order.trigger_px, Some("3000".to_string()));
        assert_eq!(order.trigger_condition, Some(TriggerCondition::Mark));
        assert_eq!(order.order_type, OrderType::Trigger);
        assert_eq!(order.is_trigger, Some(true));
    }

    #[test]
    fn test_with_cloid() {
        let order = OrderWire::new_limit(
            "BTC".to_string(),
            true,
            "0.1".to_string(),
            "50000".to_string(),
        )
        .with_cloid("my-order-123".to_string());

        assert_eq!(order.cloid, Some("my-order-123".to_string()));
        assert!(order.order_id.is_none());
    }

    #[test]
    fn test_with_oid() {
        let order = OrderWire::new_limit(
            "BTC".to_string(),
            true,
            "0.1".to_string(),
            "50000".to_string(),
        )
        .with_oid(12345);

        assert_eq!(order.order_id, Some(12345));
        assert!(order.cloid.is_none());
    }

    #[test]
    fn test_with_reduce_only() {
        let order = OrderWire::new_limit(
            "BTC".to_string(),
            true,
            "0.1".to_string(),
            "50000".to_string(),
        )
        .with_reduce_only(true);

        assert_eq!(order.reduce_only, true);
    }

    #[test]
    fn test_with_time() {
        let order = OrderWire::new_limit(
            "BTC".to_string(),
            true,
            "0.1".to_string(),
            "50000".to_string(),
        )
        .with_time(1234567890);

        assert_eq!(order.time, Some(1234567890));
    }

    #[test]
    fn test_serialization_limit_order() {
        let order = OrderWire::new_limit(
            "BTC".to_string(),
            true,
            "0.1".to_string(),
            "50000".to_string(),
        );

        let json = serde_json::to_string(&order).unwrap();
        let expected = r#"{"coin":"BTC","is_buy":true,"sz":"0.1","limitPx":"50000","reduceOnly":false,"orderType":"Limit"}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn test_serialization_with_cloid() {
        let order = OrderWire::new_limit(
            "BTC".to_string(),
            true,
            "0.1".to_string(),
            "50000".to_string(),
        )
        .with_cloid("test-123".to_string());

        let json = serde_json::to_string(&order).unwrap();
        let expected = r#"{"coin":"BTC","cloid":"test-123","is_buy":true,"sz":"0.1","limitPx":"50000","reduceOnly":false,"orderType":"Limit"}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn test_serialization_trigger_order() {
        let order = OrderWire::new_trigger(
            "ETH".to_string(),
            false,
            "1.0".to_string(),
            "3000".to_string(),
            "2990".to_string(),
            TriggerCondition::Mark,
        );

        let json = serde_json::to_string(&order).unwrap();
        let expected = r#"{"coin":"ETH","is_buy":false,"sz":"1.0","limitPx":"2990","reduceOnly":false,"orderType":"Trigger","isTrigger":true,"triggerCondition":"mark","triggerPx":"3000"}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn test_deserialization_limit_order() {
        let json = r#"{"coin":"BTC","is_buy":true,"sz":"0.1","limitPx":"50000","reduceOnly":false,"orderType":"Limit"}"#;
        let order: OrderWire = serde_json::from_str(json).unwrap();

        assert_eq!(order.coin, "BTC");
        assert_eq!(order.is_buy, true);
        assert_eq!(order.sz, "0.1");
        assert_eq!(order.limit_price, "50000");
        assert_eq!(order.reduce_only, false);
        assert_eq!(order.order_type, OrderType::Limit);
    }

    #[test]
    fn test_deserialization_with_cloid() {
        let json = r#"{"coin":"BTC","cloid":"test-123","is_buy":true,"sz":"0.1","limitPx":"50000","reduceOnly":false,"orderType":"Limit"}"#;
        let order: OrderWire = serde_json::from_str(json).unwrap();

        assert_eq!(order.coin, "BTC");
        assert_eq!(order.cloid, Some("test-123".to_string()));
        assert_eq!(order.is_buy, true);
        assert_eq!(order.sz, "0.1");
        assert_eq!(order.limit_price, "50000");
        assert_eq!(order.reduce_only, false);
        assert_eq!(order.order_type, OrderType::Limit);
    }

    #[test]
    fn test_deserialization_trigger_order() {
        let json = r#"{"coin":"ETH","is_buy":false,"sz":"1.0","limitPx":"2990","reduceOnly":false,"orderType":"Trigger","isTrigger":true,"triggerCondition":"mark","triggerPx":"3000"}"#;
        let order: OrderWire = serde_json::from_str(json).unwrap();

        assert_eq!(order.coin, "ETH");
        assert_eq!(order.is_buy, false);
        assert_eq!(order.sz, "1.0");
        assert_eq!(order.limit_price, "2990");
        assert_eq!(order.reduce_only, false);
        assert_eq!(order.order_type, OrderType::Trigger);
        assert_eq!(order.is_trigger, Some(true));
        assert_eq!(order.trigger_condition, Some(TriggerCondition::Mark));
        assert_eq!(order.trigger_px, Some("3000".to_string()));
    }
}

#[cfg(test)]
mod user_state_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_user_state_from_clearinghouse_state() {
        // Test parsing UserState from clearinghouseState response
        let clearinghouse_state_json = json!({
            "marginSummary": {
                "accountValue": "10000.0",
                "totalMarginUsed": "2000.0",
                "totalNtlPos": "5000.0",
                "totalRawUsd": "10000.0"
            },
            "crossMarginSummary": {
                "accountValue": "5000.0",
                "totalMarginUsed": "1000.0",
                "totalNtlPos": "2500.0",
                "totalRawUsd": "5000.0"
            },
            "positions": [
                {
                    "coin": "BTC",
                    "position": {
                        "szi": "0.1",
                        "entryPx": "50000.0",
                        "leverage": "5.0",
                        "liquidationPx": "45000.0",
                        "positionValue": "5000.0",
                        "marginUsed": "1000.0",
                        "openSize": "0.1",
                        "rawPNL": "100.0",
                        "returnOnEquity": "0.02",
                        "type_": "cross",
                        "userID": "12345",
                        "account": "test_account"
                    }
                },
                {
                    "coin": "ETH",
                    "position": {
                        "szi": "-0.5",
                        "entryPx": "3000.0",
                        "leverage": "3.0",
                        "liquidationPx": "3200.0",
                        "positionValue": "1500.0",
                        "marginUsed": "500.0",
                        "openSize": "0.5",
                        "rawPNL": "-50.0",
                        "returnOnEquity": "-0.01",
                        "type_": "isolated",
                        "userID": "12345",
                        "account": null
                    }
                }
            ],
            "withdrawable": "8000.0",
            "assetPositions": [
                {
                    "time": 1640995200000,
                    "token": "USDC",
                    "delta": "1000.0",
                    "deltaUsd": "1000.0",
                    "total": "9000.0",
                    "totalUsd": "9000.0",
                    "type_": "deposit"
                }
            ]
        });

        let user_state: UserState = serde_json::from_value(clearinghouse_state_json).unwrap();

        // Verify margin summary
        assert_eq!(user_state.marginSummary.accountValue, "10000.0");
        assert_eq!(user_state.marginSummary.totalMarginUsed, "2000.0");
        assert_eq!(user_state.marginSummary.totalNtlPos, "5000.0");
        assert_eq!(user_state.marginSummary.totalRawUsd, "10000.0");

        // Verify cross margin summary
        assert!(user_state.crossMarginSummary.is_some());
        let cross_margin = user_state.crossMarginSummary.as_ref().unwrap();
        assert_eq!(cross_margin.accountValue, "5000.0");
        assert_eq!(cross_margin.totalMarginUsed, "1000.0");
        assert_eq!(cross_margin.totalNtlPos, "2500.0");
        assert_eq!(cross_margin.totalRawUsd, "5000.0");

        // Verify positions
        assert_eq!(user_state.positions.len(), 2);

        // Test first position (BTC)
        let btc_position = &user_state.positions[0];
        assert_eq!(btc_position.coin, "BTC");
        assert_eq!(btc_position.position.szi, "0.1");
        assert_eq!(btc_position.position.entryPx, Some("50000.0".to_string()));
        assert_eq!(btc_position.position.leverage, Some("5.0".to_string()));
        assert_eq!(btc_position.position.liquidationPx, Some("45000.0".to_string()));
        assert_eq!(btc_position.position.positionValue, "5000.0");
        assert_eq!(btc_position.position.marginUsed, Some("1000.0".to_string()));
        assert_eq!(btc_position.position.openSize, "0.1");
        assert_eq!(btc_position.position.rawPNL, Some("100.0".to_string()));
        assert_eq!(btc_position.position.returnOnEquity, Some("0.02".to_string()));
        assert_eq!(btc_position.position.type_, "cross");
        assert_eq!(btc_position.position.userID, "12345");
        assert_eq!(btc_position.position.account, Some("test_account".to_string()));

        // Test second position (ETH)
        let eth_position = &user_state.positions[1];
        assert_eq!(eth_position.coin, "ETH");
        assert_eq!(eth_position.position.szi, "-0.5");
        assert_eq!(eth_position.position.entryPx, Some("3000.0".to_string()));
        assert_eq!(eth_position.position.leverage, Some("3.0".to_string()));
        assert_eq!(eth_position.position.liquidationPx, Some("3200.0".to_string()));
        assert_eq!(eth_position.position.positionValue, "1500.0");
        assert_eq!(eth_position.position.marginUsed, Some("500.0".to_string()));
        assert_eq!(eth_position.position.openSize, "0.5");
        assert_eq!(eth_position.position.rawPNL, Some("-50.0".to_string()));
        assert_eq!(eth_position.position.returnOnEquity, Some("-0.01".to_string()));
        assert_eq!(eth_position.position.type_, "isolated");
        assert_eq!(eth_position.position.userID, "12345");
        assert_eq!(eth_position.position.account, None);

        // Verify withdrawable
        assert_eq!(user_state.withdrawable, "8000.0");

        // Verify asset positions
        assert_eq!(user_state.assetPositions.len(), 1);
        let asset_position = &user_state.assetPositions[0];
        assert_eq!(asset_position.time, 1640995200000);
        assert_eq!(asset_position.token, "USDC");
        assert_eq!(asset_position.delta, Some("1000.0".to_string()));
        assert_eq!(asset_position.deltaUsd, Some("1000.0".to_string()));
        assert_eq!(asset_position.total, Some("9000.0".to_string()));
        assert_eq!(asset_position.totalUsd, Some("9000.0".to_string()));
        assert_eq!(asset_position.type_, Some("deposit".to_string()));
    }

    #[test]
    fn test_user_state_without_cross_margin() {
        // Test UserState without crossMarginSummary
        let user_state_json = json!({
            "marginSummary": {
                "accountValue": "15000.0",
                "totalMarginUsed": "3000.0",
                "totalNtlPos": "7500.0",
                "totalRawUsd": "15000.0"
            },
            "positions": [],
            "withdrawable": "12000.0",
            "assetPositions": []
        });

        let user_state: UserState = serde_json::from_value(user_state_json).unwrap();

        // Verify margin summary
        assert_eq!(user_state.marginSummary.accountValue, "15000.0");
        assert_eq!(user_state.marginSummary.totalMarginUsed, "3000.0");
        assert_eq!(user_state.marginSummary.totalNtlPos, "7500.0");
        assert_eq!(user_state.marginSummary.totalRawUsd, "15000.0");

        // Verify cross margin summary is None
        assert!(user_state.crossMarginSummary.is_none());

        // Verify positions is empty
        assert_eq!(user_state.positions.len(), 0);

        // Verify withdrawable
        assert_eq!(user_state.withdrawable, "12000.0");

        // Verify asset positions is empty
        assert_eq!(user_state.assetPositions.len(), 0);
    }

    #[test]
    fn test_position_details_optional_fields() {
        // Test PositionDetails with minimal required fields only
        let position_json = json!({
            "szi": "1.0",
            "positionValue": "5000.0",
            "openSize": "1.0",
            "type_": "cross",
            "userID": "12345"
        });

        let position_details: PositionDetails = serde_json::from_value(position_json).unwrap();

        // Verify required fields
        assert_eq!(position_details.szi, "1.0");
        assert_eq!(position_details.positionValue, "5000.0");
        assert_eq!(position_details.openSize, "1.0");
        assert_eq!(position_details.type_, "cross");
        assert_eq!(position_details.userID, "12345");

        // Verify optional fields are None
        assert!(position_details.entryPx.is_none());
        assert!(position_details.leverage.is_none());
        assert!(position_details.liquidationPx.is_none());
        assert!(position_details.marginUsed.is_none());
        assert!(position_details.rawPNL.is_none());
        assert!(position_details.returnOnEquity.is_none());
        assert!(position_details.account.is_none());
    }

    #[test]
    fn test_margin_summary_parsing() {
        // Test MarginSummary parsing with various numeric formats
        let margin_summary_json = json!({
            "accountValue": "10000.0",
            "totalMarginUsed": "2000.0",
            "totalNtlPos": "5000.0",
            "totalRawUsd": "10000.0"
        });

        let margin_summary: MarginSummary = serde_json::from_value(margin_summary_json).unwrap();

        assert_eq!(margin_summary.accountValue, "10000.0");
        assert_eq!(margin_summary.totalMarginUsed, "2000.0");
        assert_eq!(margin_summary.totalNtlPos, "5000.0");
        assert_eq!(margin_summary.totalRawUsd, "10000.0");
    }

    #[test]
    fn test_asset_position_parsing() {
        // Test AssetPosition parsing
        let asset_position_json = json!({
            "time": 1640995200000,
            "token": "BTC",
            "delta": "0.1",
            "deltaUsd": "5000.0",
            "total": "0.5",
            "totalUsd": "25000.0",
            "type_": "trade"
        });

        let asset_position: AssetPosition = serde_json::from_value(asset_position_json).unwrap();

        assert_eq!(asset_position.time, 1640995200000);
        assert_eq!(asset_position.token, "BTC");
        assert_eq!(asset_position.delta, Some("0.1".to_string()));
        assert_eq!(asset_position.deltaUsd, Some("5000.0".to_string()));
        assert_eq!(asset_position.total, Some("0.5".to_string()));
        assert_eq!(asset_position.totalUsd, Some("25000.0".to_string()));
        assert_eq!(asset_position.type_, Some("trade".to_string()));
    }

    #[test]
    fn test_user_state_roundtrip() {
        // Test that UserState can be serialized and deserialized correctly
        let original_user_state = UserState {
            marginSummary: MarginSummary {
                accountValue: "10000.0".to_string(),
                totalMarginUsed: "2000.0".to_string(),
                totalNtlPos: "5000.0".to_string(),
                totalRawUsd: "10000.0".to_string(),
            },
            crossMarginSummary: Some(CrossMarginSummary {
                accountValue: "5000.0".to_string(),
                totalMarginUsed: "1000.0".to_string(),
                totalNtlPos: "2500.0".to_string(),
                totalRawUsd: "5000.0".to_string(),
            }),
            positions: vec![
                Position {
                    coin: "BTC".to_string(),
                    position: PositionDetails {
                        szi: "0.1".to_string(),
                        entryPx: Some("50000.0".to_string()),
                        leverage: Some("5.0".to_string()),
                        liquidationPx: Some("45000.0".to_string()),
                        positionValue: "5000.0".to_string(),
                        marginUsed: Some("1000.0".to_string()),
                        openSize: "0.1".to_string(),
                        rawPNL: Some("100.0".to_string()),
                        returnOnEquity: Some("0.02".to_string()),
                        type_: "cross".to_string(),
                        userID: "12345".to_string(),
                        account: Some("test_account".to_string()),
                        cumFunding: None,
                        maxCost: None,
                        maxLeverage: None,
                        positionUUID: None,
                        pendingFunding: None,
                    }
                }
            ],
            withdrawable: "8000.0".to_string(),
            assetPositions: vec![
                AssetPosition {
                    time: 1640995200000,
                    token: "USDC".to_string(),
                    delta: Some("1000.0".to_string()),
                    deltaUsd: Some("1000.0".to_string()),
                    total: Some("9000.0".to_string()),
                    totalUsd: Some("9000.0".to_string()),
                    type_: Some("deposit".to_string()),
                }
            ],
        };

        // Serialize to JSON
        let json_string = serde_json::to_string(&original_user_state).unwrap();

        // Deserialize back to UserState
        let deserialized_user_state: UserState = serde_json::from_str(&json_string).unwrap();

        // Verify roundtrip equality
        assert_eq!(original_user_state.marginSummary.accountValue, deserialized_user_state.marginSummary.accountValue);
        assert_eq!(original_user_state.marginSummary.totalMarginUsed, deserialized_user_state.marginSummary.totalMarginUsed);
        assert_eq!(original_user_state.crossMarginSummary.as_ref().unwrap().accountValue, deserialized_user_state.crossMarginSummary.as_ref().unwrap().accountValue);
        assert_eq!(original_user_state.positions.len(), deserialized_user_state.positions.len());
        assert_eq!(original_user_state.positions[0].coin, deserialized_user_state.positions[0].coin);
        assert_eq!(original_user_state.positions[0].position.szi, deserialized_user_state.positions[0].position.szi);
        assert_eq!(original_user_state.withdrawable, deserialized_user_state.withdrawable);
        assert_eq!(original_user_state.assetPositions.len(), deserialized_user_state.assetPositions.len());
    }

    #[test]
    fn test_builder_info_creation() {
        // Test creating BuilderInfo with builder address and fee
        let builder_info = BuilderInfo {
            b: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            fee_in_tenths_bps: 50, // 0.05% fee
        };

        assert_eq!(builder_info.b, "0x1234567890abcdef1234567890abcdef12345678");
        assert_eq!(builder_info.fee_in_tenths_bps, 50);
    }

    #[test]
    fn test_builder_info_serialization() {
        // Test BuilderInfo serialization to wire format
        let builder_info = BuilderInfo {
            b: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            fee_in_tenths_bps: 50,
        };

        let json = serde_json::to_string(&builder_info).unwrap();
        let expected = r#"{"b":"0x1234567890abcdef1234567890abcdef12345678","f":50}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn test_builder_info_deserialization() {
        // Test BuilderInfo deserialization from JSON
        let json = r#"{"b":"0x1234567890abcdef1234567890abcdef12345678","f":50}"#;
        let builder_info: BuilderInfo = serde_json::from_str(json).unwrap();

        assert_eq!(builder_info.b, "0x1234567890abcdef1234567890abcdef12345678");
        assert_eq!(builder_info.fee_in_tenths_bps, 50);
    }

    #[test]
    fn test_builder_info_fee_validation() {
        // Test various fee values
        let builder_info_low = BuilderInfo {
            b: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            fee_in_tenths_bps: 0, // 0% fee
        };
        assert_eq!(builder_info_low.fee_in_tenths_bps, 0);

        let builder_info_high = BuilderInfo {
            b: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            fee_in_tenths_bps: 100000, // 10% fee
        };
        assert_eq!(builder_info_high.fee_in_tenths_bps, 100000);

        // Test typical fee values
        let builder_info_typical = BuilderInfo {
            b: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            fee_in_tenths_bps: 50, // 0.05% fee
        };
        assert_eq!(builder_info_typical.fee_in_tenths_bps, 50);
    }

    #[test]
    fn test_builder_info_roundtrip() {
        // Test BuilderInfo serialization and deserialization
        let original_builder_info = BuilderInfo {
            b: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            fee_in_tenths_bps: 75, // 0.075% fee
        };

        // Serialize to JSON
        let json_string = serde_json::to_string(&original_builder_info).unwrap();

        // Deserialize back to BuilderInfo
        let deserialized_builder_info: BuilderInfo = serde_json::from_str(&json_string).unwrap();

        // Verify roundtrip equality
        assert_eq!(original_builder_info.b, deserialized_builder_info.b);
        assert_eq!(original_builder_info.fee_in_tenths_bps, deserialized_builder_info.fee_in_tenths_bps);
    }
}

// Exchange API Types
// ===============================================================

/// Order type variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OrderType {
    Limit,
    Market,
    StopLimit,
    StopMarket,
    TakeProfitLimit,
    TakeProfitMarket,
}

/// Order action (for triggers)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OrderAction {
    Limit,
    Market,
}

/// Time in force
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeInForce {
    #[serde(rename = "Gtc")]
    GoodTillCanceled,
    #[serde(rename = "Ioc")]
    ImmediateOrCancel,
    #[serde(rename = "Fok")]
    FillOrKill,
    #[serde(rename = "Alo")]
    AuctionLimitOrder,
}

/// Order request for placing new orders
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderRequest {
    pub coin: String,
    pub is_buy: bool,
    pub sz: String,
    pub limit_px: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_type: Option<OrderType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_on_trigger: Option<bool>,
}

/// Response from placing orders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub status: Vec<OrderStatusResponse>,
}

/// Individual order status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatusResponse {
    pub status: String, // "ok", "error"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<ResponseDetails>,
}

/// Response details for successful orders
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseDetails {
    pub data: ResponseData,
    pub resting: bool,
    pub oid: i64,
    pub status: String,
    pub time: i64,
}

/// Response data containing order details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseData {
    pub status: Vec<ResponseStatus>,
    pub cancelStatus: Vec<ResponseStatus>,
    pub request: RequestDetails,
}

/// Response status for individual orders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseStatus {
    pub status: String, // "acknowledged", "rejected"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restings: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Request details for placed orders
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestDetails {
    pub type_: String, // "order", "cancel", "bulkCancel", "bulkCancelByMetadata"
    pub time: i64,
    pub noise: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orders: Option<Vec<OrderDetails>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancels: Option<CancelsDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelByMetadata: Option<CancelByMetadata>,
}

/// Order details for requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderDetails {
    pub coin: String,
    pub is_buy: bool,
    pub sz: String,
    pub limit_px: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_type: Option<OrderType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
}

/// Cancels details for cancel requests
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelsDetails {
    pub all: Option<bool>,
    pub oid: Option<i64>,
    pub coin: Option<String>,
    pub oids: Option<Vec<i64>>,
}

/// Cancel by metadata details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelByMetadata {
    pub metadata: String,
}

/// Cancel request for individual orders
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelRequest {
    pub coin: String,
    pub oid: i64,
}

/// Cancel by metadata request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelByMetadataRequest {
    pub coin: String,
    pub metadata: String,
}

/// Cancel all orders request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelAllRequest {
    pub coin: String,
}

/// Modify order request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifyRequest {
    pub oid: i64,
    pub order: OrderRequest,
}

/// Modify by metadata request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifyByMetadataRequest {
    pub coin: String,
    pub order: OrderRequest,
    pub metadata: String,
}

/// Transfer request for moving funds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    pub destination: String,
    pub amount: String,
    pub token: String,
}

/// Transfer response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResponse {
    pub status: String,
    pub tx_hash: Option<String>,
    pub transfer_id: Option<String>,
}

/// Update leverage request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLeverageRequest {
    pub coin: String,
    pub leverage: LeverageDetails,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_topup: Option<bool>,
}

/// Leverage details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeverageDetails {
    pub type_: LeverageType,
    pub cross_leverage: String,
}

/// Leverage type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LeverageType {
    Cross,
    Isolated,
}

/// Update margin request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMarginRequest {
    pub coin: String,
    pub type_: MarginUpdateType,
    pub amount: String,
}

/// Margin update type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarginUpdateType {
    Add,
    Remove,
}

/// Open orders request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrdersRequest {
    pub coin: String,
}

/// Open orders response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenOrdersResponse {
    pub open: Vec<OpenOrder>,
}

/// Individual open order
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrder {
    pub coin: String,
    pub limit_px: String,
    pub oid: i64,
    pub orig_sz: String,
    pub remaining_sz: String,
    pub side: String,
    pub status: String,
    pub time: i64,
    pub order_type: OrderType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
}

/// Funding history request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingHistoryRequest {
    /// Type of request
    #[serde(rename = "type")]
    pub type_: String,
    /// Coin to get funding history for
    pub coin: String,
    /// Start time in milliseconds (Unix timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    /// End time in milliseconds (Unix timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
}

/// Funding history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingHistoryResponse {
    /// Coin for which funding history is requested
    pub coin: String,
    /// List of funding payments
    pub funding_payments: Vec<FundingPayment>,
}

/// User fees information including fee tier, volume, and rates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeesResponse {
    /// Fee tier based on trading volume
    pub fee_tier: String,
    /// 30-day trading volume in USD
    #[serde(rename = "30dVolume")]
    pub volume_30d: String,
    /// Maker fee rate in basis points
    pub maker_fee: String,
    /// Taker fee rate in basis points
    pub taker_fee: String,
    /// Address for which fees are calculated
    pub address: Option<String>,
}

/// Asset context information with market data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetContext {
    /// Daily notional volume
    pub dayNtlVlm: Option<String>,
    /// Funding rate
    pub funding: Option<String>,
    /// Impact prices for liquidation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impactPxs: Option<Vec<String>>,
    /// Mark price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markPx: Option<String>,
    /// Mid price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub midPx: Option<String>,
    /// Open interest
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openInterest: Option<String>,
    /// Oracle price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oraclePx: Option<String>,
    /// Premium
    #[serde(skip_serializing_if = "Option::is_none")]
    pub premium: Option<String>,
    /// Previous day price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prevDayPx: Option<String>,
}

/// Combined meta and asset contexts response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaAndAssetContexts {
    /// Exchange metadata
    pub meta: Meta,
    /// Asset context information for all assets
    pub asset_contexts: Vec<AssetContext>,
}

impl FundingHistoryRequest {
    /// Create a new funding history request
    pub fn new(coin: String) -> Self {
        Self {
            type_: "fundingHistory".to_string(),
            coin,
            start_time: None,
            end_time: None,
        }
    }

    /// Set the start time for the funding history
    pub fn with_start_time(mut self, start_time: i64) -> Self {
        self.start_time = Some(start_time);
        self
    }

    /// Set the end time for the funding history
    pub fn with_end_time(mut self, end_time: i64) -> Self {
        self.end_time = Some(end_time);
        self
    }
}

/// Bulk order request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOrderRequest {
    pub orders: Vec<OrderRequest>,
}

/// Bulk cancel request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkCancelRequest {
    pub cancels: Vec<CancelRequest>,
}

/// Transaction signature information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSignature {
    pub sig: Signature,
    pub time: i64,
}

/// Signature details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub sig: String,
    pub signer: String,
    pub signature_type: String,
}

/// Exchange API request wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeRequest {
    pub type_: String, // "order", "cancel", "bulkCancel", etc.
    pub time: Option<i64>,
    pub nonce: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orders: Option<Vec<OrderRequest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancels: Option<Vec<CancelRequest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_by_metadata: Option<CancelByMetadataRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modify: Option<ModifyRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transfer: Option<TransferRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_leverage: Option<UpdateLeverageRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_margin: Option<UpdateMarginRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_orders: Option<OpenOrdersRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bulk_orders: Option<BulkOrderRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bulk_cancel: Option<BulkCancelRequest>,
}

#[cfg(test)]
mod exchange_api_tests {
    use super::*;

    #[test]
    fn test_order_request_serialization() {
        let order = OrderRequest {
            coin: "BTC".to_string(),
            is_buy: true,
            sz: "0.001".to_string(),
            limit_px: "50000".to_string(),
            reduce_only: Some(false),
            order_type: Some(OrderType::Limit),
            time_in_force: Some(TimeInForce::GoodTillCanceled),
            trigger_price: None,
            trail_value: None,
            close_on_trigger: None,
        };

        let json = serde_json::to_string(&order).unwrap();
        let expected = r#"{"coin":"BTC","isBuy":true,"sz":"0.001","limitPx":"50000","reduceOnly":false,"orderType":"Limit","timeInForce":"Gtc"}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn test_cancel_request_serialization() {
        let cancel = CancelRequest {
            coin: "BTC".to_string(),
            oid: 123456,
        };

        let json = serde_json::to_string(&cancel).unwrap();
        let expected = r#"{"coin":"BTC","oid":123456}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn test_transfer_request_serialization() {
        let transfer = TransferRequest {
            destination: "0x1234567890abcdef".to_string(),
            amount: "1.0".to_string(),
            token: "ETH".to_string(),
        };

        let json = serde_json::to_string(&transfer).unwrap();
        let expected = r#"{"destination":"0x1234567890abcdef","amount":"1.0","token":"ETH"}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn test_bulk_order_request_serialization() {
        let order1 = OrderRequest {
            coin: "BTC".to_string(),
            is_buy: true,
            sz: "0.001".to_string(),
            limit_px: "50000".to_string(),
            reduce_only: None,
            order_type: None,
            time_in_force: None,
            trigger_price: None,
            trail_value: None,
            close_on_trigger: None,
        };

        let bulk_request = BulkOrderRequest {
            orders: vec![order1],
        };

        let json = serde_json::to_string(&bulk_request).unwrap();
        let expected = r#"{"orders":[{"coin":"BTC","isBuy":true,"sz":"0.001","limitPx":"50000"}]}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn test_base_response_parsing() {
        // Test successful response
        let success_json = r#"{"data": {"universe": []}}"#;
        let response: ApiResponse<Meta> = serde_json::from_str(success_json).unwrap();
        assert!(response.is_success());
        assert!(!response.is_error());

        let result = response.into_result().unwrap();
        assert!(result.universe.is_empty());
    }

    #[test]
    fn test_error_response_parsing() {
        // Test error response
        let error_json = r#"{"code": 400, "msg": "Invalid request", "data": {"field": "value"}}"#;
        let response: ApiResponse<Meta> = serde_json::from_str(error_json).unwrap();
        assert!(response.is_error());
        assert!(!response.is_success());

        let error = response.error().unwrap();
        assert_eq!(error.code, 400);
        assert_eq!(error.msg, "Invalid request");
        assert!(error.data.is_some());
    }

    #[test]
    fn test_response_utils_parse_response() {
        // Test parsing success response
        let success_json = r#"{"data": {"test": "value"}}"#;
        let result: ApiResponse<serde_json::Value> = parse_response(success_json).unwrap();
        assert!(result.is_success());

        // Test parsing error response
        let error_json = r#"{"code": 500, "msg": "Server error"}"#;
        let result: ApiResponse<serde_json::Value> = parse_response(error_json).unwrap();
        assert!(result.is_error());
    }

    #[test]
    fn test_response_utils_parse_success_response() {
        // Test parsing success response to data directly
        let success_json = r#"{"data": {"test": "value"}}"#;
        let result: serde_json::Value = parse_success_response(success_json).unwrap();
        assert_eq!(result["test"], "value");

        // Test parsing error response returns error
        let error_json = r#"{"code": 400, "msg": "Bad request"}"#;
        let result: Result<serde_json::Value, ErrorResponse> = parse_success_response(error_json);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, 400);
    }

    #[test]
    fn test_response_utils_is_error_response() {
        // Test detecting error responses
        let error_json = r#"{"code": 400, "msg": "Bad request"}"#;
        assert!(is_error_response(error_json));

        // Test non-error responses
        let success_json = r#"{"data": {"test": "value"}}}"#;
        assert!(!is_error_response(success_json));

        // Test responses without error fields
        let simple_json = r#"{"status": "ok"}"#;
        assert!(!is_error_response(simple_json));
    }

    #[test]
    fn test_response_utils_extract_status() {
        // Test extracting status field
        let response_with_status = r#"{"status": "success", "data": {}}"#;
        let status = extract_status(response_with_status);
        assert_eq!(status, Some("success".to_string()));

        // Test response without status
        let response_without_status = r#"{"data": {}}"#;
        let status = extract_status(response_without_status);
        assert_eq!(status, None);
    }

    #[test]
    fn test_response_utils_extract_nested_data() {
        // Test extracting direct data field
        let response_with_data = r#"{"data": {"test": "value"}}"#;
        let result: Option<serde_json::Value> = extract_nested_data(response_with_data).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap()["test"], "value");

        // Test extracting nested data field
        let response_with_nested_data = r#"{"result": {"data": {"test": "nested"}}}"#;
        let result: Option<serde_json::Value> = extract_nested_data(response_with_nested_data).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap()["test"], "nested");

        // Test response without data field
        let response_without_data = r#"{"status": "ok"}"#;
        let result: Option<serde_json::Value> = extract_nested_data(response_without_data).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_error_response_to_hyperliquid_error() {
        let error_response = ErrorResponse {
            code: 401,
            msg: "Unauthorized".to_string(),
            data: Some(serde_json::json!({"details": "Invalid token"})),
        };

        let hyperliquid_error = error_response.to_error();
        match hyperliquid_error {
            crate::error::HyperliquidError::Client { code, message, data } => {
                assert_eq!(code, 401);
                assert_eq!(message, "Unauthorized");
                assert!(data.is_some());
            }
            _ => panic!("Expected Client error variant"),
        }
    }
}

/// Tests for OrderType enum and API compatibility
#[cfg(test)]
mod order_type_tests {
    use super::*;

    #[test]
    fn test_order_type_serialization() {
        // Test that OrderType variants serialize to correct API format
        let test_cases = vec![
            (OrderType::Limit, "Limit"),
            (OrderType::Trigger, "Trigger"),
            (OrderType::Market, "Market"),
            (OrderType::StopLimit, "StopLimit"),
            (OrderType::StopMarket, "StopMarket"),
            (OrderType::TakeProfitLimit, "TakeProfitLimit"),
            (OrderType::TakeProfitMarket, "TakeProfitMarket"),
            (OrderType::GoodTillCancel, "Gtc"),
            (OrderType::ImmediateOrCancel, "Ioc"),
            (OrderType::FillOrKill, "Fok"),
            (OrderType::AuctionLimitOrder, "Alo"),
        ];

        for (order_type, expected_json) in test_cases {
            let json = serde_json::to_string(&order_type).unwrap();
            let expected = format!("\"{}\"", expected_json);
            assert_eq!(json, expected, "Failed for variant: {:?}", order_type);
        }
    }

    #[test]
    fn test_order_type_deserialization() {
        // Test that API responses can be deserialized correctly
        let test_cases = vec![
            ("\"Limit\"", OrderType::Limit),
            ("\"Trigger\"", OrderType::Trigger),
            ("\"Market\"", OrderType::Market),
            ("\"StopLimit\"", OrderType::StopLimit),
            ("\"StopMarket\"", OrderType::StopMarket),
            ("\"TakeProfitLimit\"", OrderType::TakeProfitLimit),
            ("\"TakeProfitMarket\"", OrderType::TakeProfitMarket),
            ("\"Gtc\"", OrderType::GoodTillCancel),
            ("\"Ioc\"", OrderType::ImmediateOrCancel),
            ("\"Fok\"", OrderType::FillOrKill),
            ("\"Alo\"", OrderType::AuctionLimitOrder),
        ];

        for (json, expected) in test_cases {
            let result: OrderType = serde_json::from_str(json).unwrap();
            assert_eq!(result, expected, "Failed for JSON: {}", json);
        }
    }

    #[test]
    fn test_good_till_cancel_rename() {
        // Test the specific requirement: GoodTillCancel should serialize as "Gtc"
        let gtc = OrderType::GoodTillCancel;
        let json = serde_json::to_string(>c).unwrap();
        assert_eq!(json, "\"Gtc\"");

        // Test deserialization from "Gtc"
        let deserialized: OrderType = serde_json::from_str("\"Gtc\"").unwrap();
        assert_eq!(deserialized, OrderType::GoodTillCancel);
    }

    #[test]
    fn test_immediate_or_cancel_rename() {
        // Test ImmediateOrCancel should serialize as "Ioc"
        let ioc = OrderType::ImmediateOrCancel;
        let json = serde_json::to_string(&ioc).unwrap();
        assert_eq!(json, "\"Ioc\"");

        // Test deserialization from "Ioc"
        let deserialized: OrderType = serde_json::from_str("\"Ioc\"").unwrap();
        assert_eq!(deserialized, OrderType::ImmediateOrCancel);
    }

    #[test]
    fn test_fill_or_kill_rename() {
        // Test FillOrKill should serialize as "Fok"
        let fok = OrderType::FillOrKill;
        let json = serde_json::to_string(&fok).unwrap();
        assert_eq!(json, "\"Fok\"");

        // Test deserialization from "Fok"
        let deserialized: OrderType = serde_json::from_str("\"Fok\"").unwrap();
        assert_eq!(deserialized, OrderType::FillOrKill);
    }

    #[test]
    fn test_auction_limit_order_rename() {
        // Test AuctionLimitOrder should serialize as "Alo"
        let alo = OrderType::AuctionLimitOrder;
        let json = serde_json::to_string(&alo).unwrap();
        assert_eq!(json, "\"Alo\"");

        // Test deserialization from "Alo"
        let deserialized: OrderType = serde_json::from_str("\"Alo\"").unwrap();
        assert_eq!(deserialized, OrderType::AuctionLimitOrder);
    }

    #[test]
    fn test_camel_case_serialization() {
        // Test that camelCase variants serialize correctly
        let camel_case_variants = vec![
            (OrderType::StopLimit, "StopLimit"),
            (OrderType::StopMarket, "StopMarket"),
            (OrderType::TakeProfitLimit, "TakeProfitLimit"),
            (OrderType::TakeProfitMarket, "TakeProfitMarket"),
        ];

        for (variant, expected_name) in camel_case_variants {
            let json = serde_json::to_string(&variant).unwrap();
            let expected = format!("\"{}\"", expected_name);
            assert_eq!(json, expected);
        }
    }

    #[test]
    fn test_roundtrip_serialization() {
        // Test that all variants can be serialized and deserialized correctly
        let all_variants = vec![
            OrderType::Limit,
            OrderType::Trigger,
            OrderType::Market,
            OrderType::StopLimit,
            OrderType::StopMarket,
            OrderType::TakeProfitLimit,
            OrderType::TakeProfitMarket,
            OrderType::GoodTillCancel,
            OrderType::ImmediateOrCancel,
            OrderType::FillOrKill,
            OrderType::AuctionLimitOrder,
        ];

        for variant in all_variants {
            let json = serde_json::to_string(&variant).unwrap();
            let deserialized: OrderType = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, deserialized, "Roundtrip failed for: {:?}", variant);
        }
    }

    #[test]
    fn test_case_insensitive_deserialization() {
        // Test that deserialization is case-sensitive (as expected)
        // This ensures API compatibility - we should match exact case
        let valid_cases = vec![
            "\"limit\"",  // This should fail - not camelCase
            "\"LIMIT\"",  // This should fail - not camelCase
            "\"gtc\"",    // This should fail - not capitalized
            "\"GTC\"",    // This should fail - not "Gtc"
        ];

        for case in valid_cases {
            let result: Result<OrderType, _> = serde_json::from_str(case);
            assert!(result.is_err(), "Should fail to deserialize: {}", case);
        }
    }

    #[test]
    fn test_invalid_order_type() {
        // Test that invalid order types return appropriate errors
        let invalid_cases = vec![
            "\"InvalidType\"",
            "\"UnknownOrder\"",
            "\"\"",
            "null",
        ];

        for case in invalid_cases {
            let result: Result<OrderType, _> = serde_json::from_str(case);
            assert!(result.is_err(), "Should fail to deserialize: {}", case);
        }
    }

    #[test]
    fn test_order_type_in_struct_serialization() {
        // Test OrderType used within a struct (like OrderWire)
        #[derive(Debug, Serialize, Deserialize)]
        struct TestOrder {
            coin: String,
            order_type: OrderType,
        }

        let order = TestOrder {
            coin: "BTC".to_string(),
            order_type: OrderType::GoodTillCancel,
        };

        let json = serde_json::to_string(&order).unwrap();
        let expected = r#"{"coin":"BTC","orderType":"Gtc"}"#;
        assert_eq!(json, expected);

        // Test deserialization
        let deserialized: TestOrder = serde_json::from_str(json).unwrap();
        assert_eq!(deserialized.order_type, OrderType::GoodTillCancel);
        assert_eq!(deserialized.coin, "BTC");
    }

    #[test]
    fn test_order_type_array_serialization() {
        // Test serialization of arrays of OrderType
        let order_types = vec![
            OrderType::Limit,
            OrderType::Market,
            OrderType::GoodTillCancel,
        ];

        let json = serde_json::to_string(&order_types).unwrap();
        let expected = r#"["Limit","Market","Gtc"]"#;
        assert_eq!(json, expected);

        // Test deserialization
        let deserialized: Vec<OrderType> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, order_types);
    }

    #[test]
    fn test_order_type_with_other_fields() {
        // Test OrderType in a more complex structure
        #[derive(Debug, Serialize, Deserialize)]
        struct ComplexOrder {
            #[serde(rename = "type")]
            order_type: OrderType,
            limit_px: Option<String>,
            is_buy: bool,
        }

        let order = ComplexOrder {
            order_type: OrderType::Trigger,
            limit_px: Some("50000".to_string()),
            is_buy: true,
        };

        let json = serde_json::to_string(&order).unwrap();
        let expected = r#"{"type":"Trigger","limitPx":"50000","isBuy":true}"#;
        assert_eq!(json, expected);

        // Test deserialization
        let deserialized: ComplexOrder = serde_json::from_str(json).unwrap();
        assert_eq!(deserialized.order_type, OrderType::Trigger);
        assert_eq!(deserialized.limit_px, Some("50000".to_string()));
        assert_eq!(deserialized.is_buy, true);
    }

    #[test]
    fn test_funding_history_request_serialization() {
        // Test FundingHistoryRequest serialization
        let request = FundingHistoryRequest::new("BTC".to_string())
            .with_start_time(1640995200000)
            .with_end_time(1641081600000);

        let json = serde_json::to_string(&request).unwrap();
        let expected = r#"{"type":"fundingHistory","coin":"BTC","startTime":1640995200000,"endTime":1641081600000}"#;
        assert_eq!(json, expected);

        // Test deserialization
        let deserialized: FundingHistoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(deserialized.type_, "fundingHistory");
        assert_eq!(deserialized.coin, "BTC");
        assert_eq!(deserialized.start_time, Some(1640995200000));
        assert_eq!(deserialized.end_time, Some(1641081600000));
    }

    #[test]
    fn test_funding_history_request_minimal() {
        // Test minimal FundingHistoryRequest without time range
        let request = FundingHistoryRequest::new("ETH".to_string());

        let json = serde_json::to_string(&request).unwrap();
        let expected = r#"{"type":"fundingHistory","coin":"ETH"}"#;
        assert_eq!(json, expected);

        // Test deserialization
        let deserialized: FundingHistoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(deserialized.type_, "fundingHistory");
        assert_eq!(deserialized.coin, "ETH");
        assert_eq!(deserialized.start_time, None);
        assert_eq!(deserialized.end_time, None);
    }

    #[test]
    fn test_funding_history_response_serialization() {
        // Test FundingHistoryResponse serialization
        let response = FundingHistoryResponse {
            coin: "BTC".to_string(),
            funding_payments: vec![
                FundingPayment {
                    coin: "BTC".to_string(),
                    fundingPayment: "0.0001".to_string(),
                    type_: "fundingPayment".to_string(),
                },
                FundingPayment {
                    coin: "BTC".to_string(),
                    fundingPayment: "-0.0002".to_string(),
                    type_: "fundingPayment".to_string(),
                },
            ],
        };

        let json = serde_json::to_string(&response).unwrap();
        let expected = r#"{"coin":"BTC","fundingPayments":[{"coin":"BTC","fundingPayment":"0.0001","type":"fundingPayment"},{"coin":"BTC","fundingPayment":"-0.0002","type":"fundingPayment"}]}"#;
        assert_eq!(json, expected);

        // Test deserialization
        let deserialized: FundingHistoryResponse = serde_json::from_str(json).unwrap();
        assert_eq!(deserialized.coin, "BTC");
        assert_eq!(deserialized.funding_payments.len(), 2);
        assert_eq!(deserialized.funding_payments[0].fundingPayment, "0.0001");
        assert_eq!(deserialized.funding_payments[1].fundingPayment, "-0.0002");
    }

    #[test]
    fn test_user_fees_response_serialization() {
        // Test UserFeesResponse serialization
        let response = UserFeesResponse {
            fee_tier: "VIP1".to_string(),
            volume_30d: "1000000.0".to_string(),
            maker_fee: "2.0".to_string(),
            taker_fee: "4.0".to_string(),
            address: Some("0x1234567890abcdef".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        let expected = r#"{"feeTier":"VIP1","30dVolume":"1000000.0","makerFee":"2.0","takerFee":"4.0","address":"0x1234567890abcdef"}"#;
        assert_eq!(json, expected);

        // Test deserialization
        let deserialized: UserFeesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(deserialized.fee_tier, "VIP1");
        assert_eq!(deserialized.volume_30d, "1000000.0");
        assert_eq!(deserialized.maker_fee, "2.0");
        assert_eq!(deserialized.taker_fee, "4.0");
        assert_eq!(deserialized.address, Some("0x1234567890abcdef".to_string()));
    }
}

#[cfg(test)]
mod staking_tests {
    use super::*;

    #[test]
    fn test_staking_summary_serialization() {
        let summary = StakingSummary {
            total_delegated: "100.0".to_string(),
            total_pending_rewards: "5.0".to_string(),
            delegation_count: 3,
            total_earned_rewards: "15.0".to_string(),
        };

        let json = serde_json::to_string(&summary).unwrap();
        let expected = r#"{"totalDelegated":"100.0","totalPendingRewards":"5.0","delegationCount":3,"totalEarnedRewards":"15.0"}"#;
        assert_eq!(json, expected);

        let deserialized: StakingSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_delegated, "100.0");
        assert_eq!(deserialized.delegation_count, 3);
    }

    #[test]
    fn test_delegation_serialization() {
        let delegation = Delegation {
            validator_address: "0x1234567890abcdef".to_string(),
            amount: "50.0".to_string(),
            pending_rewards: "2.5".to_string(),
            status: "active".to_string(),
            delegated_at: 1640995200000,
            last_claimed_at: Some(1641081600000),
        };

        let json = serde_json::to_string(&delegation).unwrap();
        let expected = r#"{"validatorAddress":"0x1234567890abcdef","amount":"50.0","pendingRewards":"2.5","status":"active","delegatedAt":1640995200000,"lastClaimedAt":1641081600000}"#;
        assert_eq!(json, expected);

        let deserialized: Delegation = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.validator_address, "0x1234567890abcdef");
        assert_eq!(deserialized.status, "active");
    }

    #[test]
    fn test_reward_event_serialization() {
        let event = RewardEvent {
            event_type: RewardEventType::Claimed,
            validator_address: "0x1234567890abcdef".to_string(),
            amount: "1.5".to_string(),
            timestamp: 1641081600000,
            tx_hash: Some("0xabcdef1234567890".to_string()),
        };

        let json = serde_json::to_string(&event).unwrap();
        let expected = r#"{"eventType":"claimed","validatorAddress":"0x1234567890abcdef","amount":"1.5","timestamp":1641081600000,"txHash":"0xabcdef1234567890"}"#;
        assert_eq!(json, expected);

        let deserialized: RewardEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.event_type, RewardEventType::Claimed);
        assert_eq!(deserialized.amount, "1.5");
    }

    #[test]
    fn test_delegator_event_serialization() {
        let event = DelegatorEvent {
            event_type: DelegatorEventType::Delegated,
            validator_address: "0x1234567890abcdef".to_string(),
            amount: "25.0".to_string(),
            timestamp: 1640995200000,
            tx_hash: Some("0x1234567890abcdef".to_string()),
            status: "completed".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let expected = r#"{"eventType":"delegated","validatorAddress":"0x1234567890abcdef","amount":"25.0","timestamp":1640995200000,"txHash":"0x1234567890abcdef","status":"completed"}"#;
        assert_eq!(json, expected);

        let deserialized: DelegatorEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.event_type, DelegatorEventType::Delegated);
        assert_eq!(deserialized.status, "completed");
    }

    #[test]
    fn test_validator_info_serialization() {
        let validator = ValidatorInfo {
            address: "0x1234567890abcdef".to_string(),
            name: "Test Validator".to_string(),
            commission_rate: "0.05".to_string(),
            total_staked: "1000.0".to_string(),
            delegator_count: 50,
            status: "active".to_string(),
            description: Some("A test validator".to_string()),
            website: Some("https://test-validator.com".to_string()),
            created_at: 1640995200000,
        };

        let json = serde_json::to_string(&validator).unwrap();
        let expected = r#"{"address":"0x1234567890abcdef","name":"Test Validator","commissionRate":"0.05","totalStaked":"1000.0","delegatorCount":50,"status":"active","description":"A test validator","website":"https://test-validator.com","createdAt":1640995200000}"#;
        assert_eq!(json, expected);

        let deserialized: ValidatorInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Test Validator");
        assert_eq!(deserialized.delegator_count, 50);
    }

    #[test]
    fn test_staking_config_serialization() {
        let config = StakingConfig {
            min_delegation: "1.0".to_string(),
            max_validators_per_user: 10,
            unbonding_period_ms: 86400000, // 24 hours
            reward_distribution_interval: 3600000, // 1 hour
        };

        let json = serde_json::to_string(&config).unwrap();
        let expected = r#"{"minDelegation":"1.0","maxValidatorsPerUser":10,"unbondingPeriodMs":86400000,"rewardDistributionInterval":3600000}"#;
        assert_eq!(json, expected);

        let deserialized: StakingConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.min_delegation, "1.0");
        assert_eq!(deserialized.max_validators_per_user, 10);
    }
}

/// Portfolio performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Portfolio {
    /// User address
    pub user: String,
    /// Current portfolio value
    pub portfolio_value: String,
    /// Account value history across different time periods
    pub account_value_history: AccountValueHistory,
    /// PnL history across different time periods
    pub pnl_history: PnlHistory,
    /// Trading volume metrics
    pub volume_metrics: VolumeMetrics,
    /// Portfolio breakdown by asset
    pub asset_breakdown: Vec<AssetAllocation>,
    /// Timestamp of data (Unix milliseconds)
    pub timestamp: i64,
}

/// Account value history across different time periods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountValueHistory {
    /// Account value 1 hour ago
    pub one_hour_ago: String,
    /// Account value 1 day ago
    pub one_day_ago: String,
    /// Account value 1 week ago
    pub one_week_ago: String,
    /// Account value 1 month ago
    pub one_month_ago: String,
    /// Account value 3 months ago
    pub three_months_ago: String,
    /// Account value 6 months ago
    pub six_months_ago: String,
    /// Account value 1 year ago
    pub one_year_ago: String,
}

/// PnL history across different time periods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PnlHistory {
    /// PnL over the last hour
    pub one_hour_pnl: String,
    /// PnL over the last day
    pub one_day_pnl: String,
    /// PnL over the last week
    pub one_week_pnl: String,
    /// PnL over the last month
    pub one_month_pnl: String,
    /// PnL over the last 3 months
    pub three_months_pnl: String,
    /// PnL over the last 6 months
    pub six_months_pnl: String,
    /// PnL over the last year
    pub one_year_pnl: String,
    /// Total PnL since account creation
    pub total_pnl: String,
}

/// Trading volume metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeMetrics {
    /// Volume in the last hour
    pub one_hour_volume: String,
    /// Volume in the last day
    pub one_day_volume: String,
    /// Volume in the last week
    pub one_week_volume: String,
    /// Volume in the last month
    pub one_month_volume: String,
    /// Total volume since account creation
    pub total_volume: String,
    /// Number of trades in the last day
    pub daily_trade_count: i64,
    /// Average trade size
    pub average_trade_size: String,
}

/// Asset allocation breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetAllocation {
    /// Asset symbol
    pub symbol: String,
    /// Allocation percentage (0-100)
    pub allocation: String,
    /// Current value in USD
    pub value_usd: String,
    /// Current quantity
    pub quantity: String,
    /// PnL for this asset
    pub pnl: String,
    /// PnL percentage for this asset
    pub pnl_percentage: String,
}

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