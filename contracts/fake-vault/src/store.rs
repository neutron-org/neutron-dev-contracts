use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::{Item, Map};

pub const USERS: Map<String, u64> = Map::new("users");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub dao: String,
    pub description: String,
    pub info: String,
    pub name: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
