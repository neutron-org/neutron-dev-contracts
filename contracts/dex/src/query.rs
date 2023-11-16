use cosmwasm_std::{Binary, Coin, Int128, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use crate::msg::{MultiHopRoute};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DexQuery {
    // Parameters queries the parameters of the module.
    Params {},
    // Queries a LimitOrderTrancheUser by index.
    LimitOrderTrancheUser {
        address: String,
        tranche_key: String,
    },
    // Queries a list of LimitOrderTrancheMap items.
    LimitOrderTrancheUserAll {
        // pagination: Option<PageRequest>,
    },
    // Queries a list of LimitOrderTrancheUser items for a given address.
    LimitOrderTrancheUserAllByAddress {
        address: String,
        // pagination: Option<PageRequest>,
    },
    // Queries a LimitOrderTranche by index.
    LimitOrderTranche {
        pair_id: String,
        tick_index: i64,
        token_in: String,
        tranche_key: String,
    },
    // Queries a list of LimitOrderTranche items for a given pairID / TokenIn combination.
    LimitOrderTrancheAll {
        pair_id: String,
        token_in: String,
        // pagination: Option<PageRequest>,
    },
    // Queries a list of UserDeposits items.
    UserDepositAll {
        address: String,
        // pagination: Option<PageRequest>,
    },
    // Queries a list of TickLiquidity items.
    TickLiquidityAll {
        pair_id: String,
        token_in: String,
        // pagination: Option<PageRequest>,
    },
    // Queries a InactiveLimitOrderTranche by index.
    InactiveLimitOrderTranche {
        pair_id: String,
        tick_index: i64,
        token_in: String,
        tranche_key: String,
    },
    // Queries a list of InactiveLimitOrderTranche items.
    InactiveLimitOrderTrancheAll {
        // pagination: Option<PageRequest>,
    },
    // Queries a list of PoolReserves items.
    PoolReservesAll {
        pair_id: String,
        token_in: String,
        // pagination: Option<PageRequest>,
    },
    // Queries a PoolReserve by index
    PoolReserves {
        pair_id: String,
        token_in: String,
        tick_index: i64,
        fee: u64,
    },
    // Queries the simulated result of a multihop swap
    EstimateMultiHopSwap {
        creator: String,
        receiver: String,
        routes: Vec<MultiHopRoute>,
        amount_in: String,
        exit_limit_price: String,
        pick_best_route: bool,
    },
    // // Queries the simulated result of a PlaceLimit order
    EstimatePlaceLimitOrder {
        creator: String,
        receiver: String,
        token_in: String,
        token_out: String,
        tick_index_in_to_out: Uint128,
        // order_type: LimitOrderType,
        // expirationTime is only valid iff orderType == GOOD_TIL_TIME.
        expiration_time: Option<u64>,
        max_amount_out: Option<Int128>,
    },
    // Queries a pool by pair, tick and fee
    Pool {
        pair_id: String,
        tick_index: i64,
        fee: u64,
    },
    // Queries a pool by ID
    #[serde(rename = "pool_by_id")]
    PoolByID {
        pool_id: u64,
    },
    // Queries a PoolMetadata by ID
    PoolMetadata {
        id: u64,
    },
    // Queries a list of PoolMetadata items.
    PoolMetadataAll {
        // pagination: Option<PageRequest>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Params {
    fee_tiers: Vec<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ParamsResponse {
    params: Params,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LimitOrderTrancheUserResponse {
    limit_order_tranche_user: Option<LimitOrderTrancheUser>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AllLimitOrderTrancheUserResponse {
    #[serde(default)]
    limit_order_tranche_user: Vec<LimitOrderTrancheUser>,
    pagination: Option<PageResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub struct AllUserLimitOrdersResponse {
    #[serde(default)]
    limit_orders: Vec<LimitOrderTrancheUser>,
    pagination: Option<PageResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LimitOrderTrancheResponse {
    limit_order_tranche: Option<LimitOrderTranche>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub struct AllLimitOrderTrancheResponse {
    #[serde(default)]
    limit_order_tranche: Vec<LimitOrderTranche>,
    pagination: Option<PageResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub struct AllUserDepositsResponse {
    #[serde(default)]
    deposits: Vec<DepositRecord>,
    pagination: Option<PageResponse>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DepositRecord {
    pair_id: PairID,
    shares_owned: String,
    center_tick_index: i64,
    lower_tick_index: i64,
    upper_tick_index: i64,
    fee: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PairID {
    token0: String,
    token1: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AllTickLiquidityResponse {
    tick_liquidity: Vec<TickLiquidity>,
    pagination: Option<PageResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TickLiquidity {
    #[serde(rename = "Liquidity")]
    liquidity: Liquidity,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Liquidity {
    PoolReserves(PoolReserves),
    LimitOrderTranche(LimitOrderTranche),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PoolReserves {
    key: PoolReservesKey,
    reserves_maker_denom: String,
    price_taker_to_maker: String,
    price_opposite_taker_to_maker: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PoolReservesKey {
    trade_pair_id: TradePairID,
    tick_index_taker_to_maker: i64,
    fee: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InactiveLimitOrderTrancheResponse {
    inactive_limit_order_tranche: LimitOrderTranche,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AllInactiveLimitOrderTrancheResponse {
    inactive_limit_order_tranche: Vec<LimitOrderTranche>,
    pagination: Option<PageResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AllPoolReservesResponse {
    pool_reserves: Vec<PoolReserves>,
    pagination: Option<PageResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PoolReservesResponse {
    pool_reserves: PoolReserves,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct EstimateMultiHopSwapResponse {
    coin_out: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct EstimatePlaceLimitOrderResponse {
    // Total amount of coin used for the limit order
    // You can derive makerLimitInCoin using the equation: totalInCoin = swapInCoin + makerLimitInCoin
    total_in_coin: Coin,
    // Total amount of the token in that was immediately swapped for swapOutCoin
    swap_in_coin: Coin,
    // Total amount of coin received from the taker portion of the limit order
    // This is the amount of coin immediately available in the users account after executing the
    // limit order. It does not include any future proceeds from the maker portion which will have withdrawn in the future
    swap_out_coin: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PoolResponse {
    pool: Pool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Pool {
    id: u64,
    lower_tick0: Option<PoolReserves>,
    lower_tick1: Option<PoolReserves>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PoolMetadataResponse {
    pool_metadata: PoolMetadata,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PoolMetadata {
    id: u64,
    tick: i64,
    fee: u64,
    pair_id: PairID,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AllPoolMetadataResponse {
    pool_metadata: Vec<PoolMetadata>,
    pagination: Option<PageResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PageResponse {
    /// **next_key** is the key to be passed to PageRequest.key to
    /// query the next page most efficiently. It will be empty if
    /// there are no more results.
    pub next_key: Option<Binary>,
    /// **total** is total number of results available if PageRequest.count_total
    /// was set, its value is undefined otherwise
    pub total: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct QueryFailuresResponse {
}

#[derive(Serialize_repr, Deserialize_repr, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[repr(u8)]
pub enum LimitOrderType {
    GoodTilCancelled = 0,
    FillOrKill = 1,
    ImmediateOrCancel = 2,
    JustInTime = 3,
    GoodTilTime = 4,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LimitOrderTrancheUser {
    trade_pair_id: TradePairID,
    tick_index_taker_to_maker: i64,
    tranche_key: String,
    address: String,
    shares_owned: Int128,
    shares_withdrawn: Int128,
    shares_cancelled: Int128,
    order_type: LimitOrderType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LimitOrderTrancheKey {
    pub trade_pair_id: TradePairID,
    pub tick_index_taker_to_maker: i64,
    pub tranche_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LimitOrderTranche {
    pub key: LimitOrderTrancheKey,
    reserves_maker_denom: Int128,
    reserves_taker_denom: Int128,
    total_maker_denom: Int128,
    total_taker_denom: Int128,
    expiration_time: Option<u64>,
    price_taker_to_maker: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TradePairID {
    maker_denom: String,
    taker_denom: String,
}