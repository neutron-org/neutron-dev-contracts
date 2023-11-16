use cosmwasm_std::Int128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use crate::msg::PrecDec;

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
    shares_owned: String,
    shares_withdrawn: String,
    shares_cancelled: String,
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
    reserves_maker_denom: String,
    reserves_taker_denom: String,
    total_maker_denom: String,
    total_taker_denom: String,
    expiration_time: Option<u64>,
    price_taker_to_maker: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TradePairID {
    maker_denom: String,
    taker_denom: String,
}