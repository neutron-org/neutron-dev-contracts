use cosmwasm_std::Coin;
use cw_storage_plus::Map;

/// Contains all submitted undelegations done by an address.
pub const UNDELEGATED_AMOUNTS: Map<String, Vec<Coin>> = Map::new("undelegated_amounts");
