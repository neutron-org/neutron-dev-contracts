use cosmwasm_std::Uint128;
use neutron_std::types::cosmos::bank::v1beta1::DenomUnit;
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
        mint_to_address: Option<String>,
    },
    BurnTokens {
        denom: String,
        amount: Uint128,
        burn_from_address: Option<String>,
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
    FullDenom { creator: String, subdenom: String },
    DenomAdmin { creator: String, subdenom: String },
    BeforeSendHook { creator: String, subdenom: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}
