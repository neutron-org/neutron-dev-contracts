use neutron_std::shim::Timestamp;
use neutron_std::types::cosmos::base::query::v1beta1::PageRequest;
use neutron_std::types::neutron::dex::{
    DepositOptions, MsgCancelLimitOrder, MsgDeposit, MsgMultiHopSwap, MsgPlaceLimitOrder,
    MsgWithdrawFilledLimitOrder, MsgWithdrawal, MultiHopRoute,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Deposit {
        receiver: String,
        token_a: String,
        token_b: String,
        amounts_a: Vec<String>,
        amounts_b: Vec<String>,
        tick_indexes_a_to_b: Vec<i64>,
        fees: Vec<u64>,
        options: Vec<DepositOptions>,
    },
    Withdrawal {
        receiver: String,
        token_a: String,
        token_b: String,
        shares_to_remove: Vec<String>,
        tick_indexes_a_to_b: Vec<i64>,
        fees: Vec<u64>,
    },
    PlaceLimitOrder {
        receiver: String,
        token_in: String,
        token_out: String,
        tick_index_in_to_out: i64,
        amount_in: String,
        order_type: i32,
        expiration_time: Option<Timestamp>,
        max_amount_out: Option<String>,
        limit_sell_price: String,
    },
    WithdrawFilledLimitOrder {
        tranche_key: String,
    },
    CancelLimitOrder {
        tranche_key: String,
    },
    MultiHopSwap {
        receiver: String,
        routes: Vec<MultiHopRoute>,
        amount_in: String,
        exit_limit_price: String,
        pick_best_route: bool,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Params {},
    GetLimitOrderTrancheUser {
        address: String,
        tranche_key: String,
        calc_withdrawable_shares: bool,
    },
    AllLimitOrderTrancheUser {
        pagination: Option<PageRequest>,
    },
    AllLimitOrderTrancheUserByAddress {
        address: String,
        pagination: Option<PageRequest>,
    },
    GetLimitOrderTranche {
        pair_id: String,
        tick_index: i64,
        token_in: String,
        tranche_key: String,
    },
    AllLimitOrderTranche {
        pair_id: String,
        token_in: String,
        pagination: Option<PageRequest>,
    },
    AllUserDeposits {
        address: String,
        include_pool_data: bool,
        pagination: Option<PageRequest>,
    },
    AllTickLiquidity {
        pair_id: String,
        token_in: String,
        pagination: Option<PageRequest>,
    },
    GetInactiveLimitOrderTranche {
        pair_id: String,
        token_in: String,
        tick_index: i64,
        tranche_key: String,
    },
    AllInactiveLimitOrderTranche {
        pagination: Option<PageRequest>,
    },
    AllPoolReserves {
        pair_id: String,
        token_in: String,
        pagination: Option<PageRequest>,
    },
    GetPoolReserves {
        pair_id: String,
        token_in: String,
        tick_index: i64,
        fee: u64,
    },
    EstimateMultiHopSwap {
        creator: String,
        receiver: String,
        routes: Vec<MultiHopRoute>,
        amount_in: String,
        exit_limit_price: String,
        pick_best_route: bool,
    },
    EstimatePlaceLimitOrder {
        creator: String,
        receiver: String,
        token_in: String,
        token_out: String,
        tick_index_in_to_out: i64,
        amount_in: String,
        order_type: i32,
        expiration_time: Option<Timestamp>,
        max_amount_out: String,
    },
    Pool {
        pair_id: String,
        tick_index: i64,
        fee: u64,
    },
    PoolById {
        pool_id: u64,
    },
    GetPoolMetadata {
        id: u64,
    },
    AllPoolMetadata {
        pagination: Option<PageRequest>,
    },
    SimulateDeposit {
        msg: MsgDeposit,
    },
    SimulateWithdrawal {
        msg: MsgWithdrawal,
    },
    SimulatePlaceLimitOrder {
        msg: MsgPlaceLimitOrder,
    },
    SimulateWithdrawFilledLimitOrder {
        msg: MsgWithdrawFilledLimitOrder,
    },
    SimulateCancelLimitOrder {
        msg: MsgCancelLimitOrder,
    },
    SimulateMultiHopSwap {
        msg: MsgMultiHopSwap,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}
