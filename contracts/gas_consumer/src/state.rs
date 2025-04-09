use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub owner: String,
    pub hash_iterations: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
