use cosmwasm_std::Coin;
use cw_storage_plus::{Item, Map};
use neutron_sdk::bindings::msg::IbcFee;

pub const IBC_FEE: Item<IbcFee> = Item::new("ibc_fee");
pub const TEST_ARGS: Map<&str, (String, Vec<Coin>)> = Map::new("test_args");
