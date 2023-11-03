use cosmwasm_std::Coin;
use neutron_sdk::bindings::msg::UnstakeDescriptor;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddToGauge {
        gauge_id: u64,
        rewards: Vec<Coin>,
    },
    Stake {
        coins: Vec<Coin>,
    },
    Unstake {
        unstakes: Vec<UnstakeDescriptor>
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    ModuleState {},
    GaugeByID { id: u64 },
    Gauges { status: String, denom: String },
    StakeByID { stake_id: u64 },
    Stakes { owner: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}
