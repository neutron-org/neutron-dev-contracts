use cosmwasm_std::{from_binary, to_vec, Binary, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use neutron_sdk::bindings::msg::IbcFee;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const IBC_FEE: Item<IbcFee> = Item::new("ibc_fee");
