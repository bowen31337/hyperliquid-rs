"""Type definitions for Hyperliquid SDK"""

from enum import Enum
from typing import Optional

from pydantic import BaseModel, Field


class AssetMeta(BaseModel):
    """Asset metadata"""
    name: str
    onlyIsolated: bool
    szDecimals: int
    maxLeverage: int
    maxDynamicLeverage: Optional[int] = None
    type: Optional[str] = None
    tokens: Optional[list["AssetMeta"]] = None
    maxOi: Optional[str] = None
    underlying: Optional[str] = None
    isInverse: Optional[bool] = None


class MetaResponse(BaseModel):
    """Response for meta endpoint"""
    universe: list[AssetMeta]


class OrderType(str, Enum):
    """Order types"""
    LIMIT = "Limit"
    TRIGGER = "Trigger"


class TriggerCondition(str, Enum):
    """Trigger conditions"""
    MARK = "mark"
    INDEX = "index"
    LAST = "last"


class PegPriceType(str, Enum):
    """Peg price types"""
    MID = "Mid"
    ORACLE = "Oracle"
    LAST = "Last"
    OPPOSITE = "Opposite"
    ORACLE_WITH_FALLBACK = "OracleWithFallback"


class OrderWire(BaseModel):
    """Order wire format for API serialization"""

    # Basic order fields
    coin: str
    is_buy: bool
    sz: str
    limitPx: str

    # Optional fields
    cloid: Optional[str] = None
    oid: Optional[int] = None
    reduceOnly: bool = False
    orderType: OrderType = OrderType.LIMIT

    # Pegged order fields
    pegOffsetValue: Optional[str] = None
    pegPriceType: Optional[PegPriceType] = None

    # Trigger order fields
    isTrigger: Optional[bool] = None
    triggerCondition: Optional[TriggerCondition] = None
    triggerPx: Optional[str] = None

    # Additional fields
    time: Optional[int] = None
    type_: Optional[str] = None
    coinOrderOpt: Optional[str] = None
    isPositionTpsl: Optional[bool] = None


class Order(BaseModel):
    """High-level Order model for Python users

    This provides a more user-friendly interface compared to OrderWire,
    with automatic type conversion and validation.
    """

    # Core order fields
    coin: str = Field(description="Trading pair (e.g., 'BTC', 'ETH')")
    is_buy: bool = Field(description="True for buy order, False for sell order")
    size: float = Field(gt=0, description="Order quantity")
    limit_price: float = Field(gt=0, description="Limit price")

    # Order configuration
    order_type: OrderType = Field(default=OrderType.LIMIT, description="Order type")
    reduce_only: bool = Field(default=False, description="Reduce-only order")
    client_order_id: Optional[str] = Field(default=None, description="Client order ID")

    # Pegged order configuration
    peg_offset_value: Optional[float] = Field(default=None, description="Peg offset value")
    peg_price_type: Optional[PegPriceType] = Field(default=None, description="Peg price type")

    # Trigger order configuration
    is_trigger: bool = Field(default=False, description="Trigger order")
    trigger_condition: Optional[TriggerCondition] = Field(default=None, description="Trigger condition")
    trigger_price: Optional[float] = Field(default=None, description="Trigger price")

    class Config:
        """Pydantic configuration"""
        allow_population_by_field_name = True
        json_encoders = {
            float: lambda v: f"{v:.15g}"  # Convert float to string with maximum precision
        }

    def to_wire(self) -> OrderWire:
        """Convert to OrderWire format for API serialization

        Returns:
            OrderWire: API-compatible order format
        """
        # Convert float fields to string format expected by API
        sz_str = f"{self.size:.15g}"
        limit_px_str = f"{self.limit_price:.15g}"

        # Convert optional float fields
        peg_offset_str = f"{self.peg_offset_value:.15g}" if self.peg_offset_value is not None else None
        trigger_px_str = f"{self.trigger_price:.15g}" if self.trigger_price is not None else None

        return OrderWire(
            coin=self.coin,
            is_buy=self.is_buy,
            sz=sz_str,
            limitPx=limit_px_str,
            orderType=self.order_type,
            reduceOnly=self.reduce_only,
            cloid=self.client_order_id,
            pegOffsetValue=peg_offset_str,
            pegPriceType=self.peg_price_type,
            isTrigger=self.is_trigger,
            triggerCondition=self.trigger_condition,
            triggerPx=trigger_px_str
        )

    @classmethod
    def from_wire(cls, wire: OrderWire) -> "Order":
        """Create Order from OrderWire format

        Args:
            wire: OrderWire instance from API response

        Returns:
            Order: High-level Order instance
        """
        return cls(
            coin=wire.coin,
            is_buy=wire.is_buy,
            size=float(wire.sz),
            limit_price=float(wire.limitPx),
            order_type=wire.orderType,
            reduce_only=wire.reduceOnly,
            client_order_id=wire.cloid,
            peg_offset_value=float(wire.pegOffsetValue) if wire.pegOffsetValue else None,
            peg_price_type=wire.pegPriceType,
            is_trigger=wire.isTrigger or False,
            trigger_condition=wire.triggerCondition,
            trigger_price=float(wire.triggerPx) if wire.triggerPx else None
        )


class NewOrder(BaseModel):
    """New order request"""

    coin: str
    is_buy: bool
    sz: str
    limitPx: str
    orderType: OrderType = OrderType.LIMIT
    reduceOnly: bool = False
    cloid: Optional[str] = None
    time: Optional[int] = None
    isTrigger: Optional[bool] = None
    triggerCondition: Optional[TriggerCondition] = None
    triggerPx: Optional[str] = None
    pegOffsetValue: Optional[str] = None
    pegPriceType: Optional[PegPriceType] = None


class OrderStatus(BaseModel):
    """Order status response"""

    coin: str
    oid: Optional[int] = None
    status: str
    type_: str
    cloid: Optional[str] = None


class MarginSummary(BaseModel):
    """Margin summary"""
    accountValue: str
    totalMarginUsed: str
    totalNtlPos: str
    totalRawUsd: str


class PositionDetails(BaseModel):
    """Position details"""
    szi: str
    entryPx: Optional[str] = None
    leverage: Optional[str] = None
    liquidationPx: Optional[str] = None
    positionValue: str
    marginUsed: Optional[str] = None
    openSize: str
    rawPNL: Optional[str] = None
    returnOnEquity: Optional[str] = None
    type: str
    userID: str
    account: Optional[str] = None
    cumFunding: Optional[str] = None
    maxCost: Optional[str] = None
    maxLeverage: Optional[str] = None
    positionUUID: Optional[str] = None
    pendingFunding: Optional[str] = None


class Position(BaseModel):
    """Position"""
    coin: str
    position: PositionDetails


class AssetPosition(BaseModel):
    """Asset position details"""
    time: int
    token: str
    delta: Optional[str] = None
    deltaUsd: Optional[str] = None
    total: Optional[str] = None
    totalUsd: Optional[str] = None
    type_: Optional[str] = None


class UserStateResponse(BaseModel):
    """Response for user state endpoint"""
    marginSummary: MarginSummary
    crossMarginSummary: Optional[MarginSummary] = None
    positions: list[Position]
    withdrawable: str
    assetPositions: list[AssetPosition] = []


# Staking Types
# ===============================================================

class StakingSummary(BaseModel):
    """Staking summary for a user including total delegated and rewards"""
    total_delegated: str
    total_pending_rewards: str
    delegation_count: int
    total_earned_rewards: str


class Delegation(BaseModel):
    """Individual delegation to a validator"""
    validator_address: str
    amount: str
    pending_rewards: str
    status: str
    delegated_at: int
    last_claimed_at: Optional[int] = None


class RewardEventType(str, Enum):
    """Type of reward event"""
    ACCRUED = "Accrued"
    CLAIMED = "Claimed"
    DELEGATED = "Delegated"
    UNDELEGATED = "Undelegated"


class RewardEvent(BaseModel):
    """Individual reward event (claim or accrual)"""
    event_type: RewardEventType
    validator_address: str
    amount: str
    timestamp: int
    tx_hash: Optional[str] = None


class StakingRewards(BaseModel):
    """Staking rewards history"""
    total_claimed: str
    total_pending: str
    history: list[RewardEvent]


class DelegatorEventType(str, Enum):
    """Type of delegator event"""
    DELEGATED = "Delegated"
    DELEGATED_MORE = "DelegatedMore"
    UNDELEGATED = "Undelegated"
    UNDELEGATED_ALL = "UndelegatedAll"
    REWARDS_CLAIMED = "RewardsClaimed"
    SLASHED = "Slashed"


class DelegatorEvent(BaseModel):
    """Individual delegator event"""
    event_type: DelegatorEventType
    validator_address: str
    amount: str
    timestamp: int
    tx_hash: Optional[str] = None
    status: str


class DelegatorSummary(BaseModel):
    """Delegator summary statistics"""
    total_delegated_lifetime: str
    total_rewards_lifetime: str
    total_slashed_lifetime: str
    current_delegation_count: int
    first_delegation_at: Optional[int] = None
    last_activity_at: int


class DelegatorHistory(BaseModel):
    """Comprehensive delegator history including all events"""
    events: list[DelegatorEvent]
    summary: DelegatorSummary
