use cosmwasm_std::{DenomUnit, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateDenom {
        subdenom: String,
    },
    ChangeAdmin {
        denom: String,
        new_admin_address: String,
    },
    MintTokens {
        denom: String,
        amount: Uint128,
    },
    BurnTokens {
        denom: String,
        amount: Uint128,
    },
    SendTokens {
        recipient: String,
        denom: String,
        amount: Uint128,
    },
    SetBeforeSendHook {
        denom: String,
        contract_addr: String,
    },
    ForceTransfer {
        denom: String,
        amount: Uint128,
        from: String,
        to: String,
    },
    SetDenomMetadata {
        description: String,
        denom_units: Vec<DenomUnit>,
        base: String,
        display: String,
        name: String,
        symbol: String,
        uri: String,
        uri_hash: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    FullDenom {
        creator_addr: String,
        subdenom: String,
    },
    DenomAdmin {
        subdenom: String,
    },
    BeforeSendHook {
        denom: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}
