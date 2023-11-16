use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use neutron_sdk::proto_types::neutron::dex::MultiHopRoute as MHR;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum DexMsg {
    Deposit {
        receiver: String,
        token_a: String,
        token_b: String,
        amounts_a: Vec<String>,
        amounts_b: Vec<String>,
        tick_indexes_a_to_b: Vec<i64>,
        fees: Vec<u64>,
        options: Vec<Option<DepositOption>>,
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
        // TODO: fix time representation
        // expirationTime is only valid iff orderType == GOOD_TIL_TIME.
        // expiration_time: Option<prost_types::Timestamp>,
        max_amount_out: String,
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
pub struct DepositOption {
    disable_swap: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MultiHopRoute {
    hops: Vec<String>,
}

impl Into<MHR> for &MultiHopRoute {
    fn into(self) -> MHR {
        MHR{ hops: self.hops.clone() }
    }
}