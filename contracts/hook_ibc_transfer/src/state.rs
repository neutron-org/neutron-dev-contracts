use cosmwasm_std::Coin;
use cw_storage_plus::{Item, Map};
use neutron_sdk::bindings::msg::IbcFee;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const IBC_FEE: Item<IbcFee> = Item::new("ibc_fee");
pub const TEST_ARGS: Map<&str, (String, Vec<Coin>)> = Map::new("test_args");
pub const IBC_TEST_ACKS: Item<IbcTestAck> = Item::new("ibc_test_acks");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IbcTestAck {
    Response(bool),
    Timeout,
}
