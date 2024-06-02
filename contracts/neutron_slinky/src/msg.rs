// use crate::storage::{AcknowledgementResult, IntegrationTestsSudoFailureMock};
use cosmwasm_std::Uint64;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// this query goes to neutron and get stored ICA with a specific query
    GetPrice {
        base_symbol: String,
        quote_currency: String,
        max_blocks_old: Uint64,
    },
    // this query returns ICA from contract store, which saved from acknowledgement
    GetPrices {
        base_symbols: Vec<String>,
        quote_currency: String,
        max_blocks_old: Uint64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {}
