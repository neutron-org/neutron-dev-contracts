use cosmwasm_std::Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Send {
        channel: String,
        to: String,
        denom: String,
        amount: u128,
        timeout_height: Option<u64>,
        memo: Option<String>,
    },
    SetFees {
        recv_fee: u128,
        ack_fee: u128,
        timeout_fee: u128,
        denom: String,
    },
    // This message is for checking result fields in integration tests
    TestMsg {
        return_err: bool,
        arg: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    TestMsg { arg: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TestArgResponse {
    pub sender: String,
    pub funds: Vec<Coin>,
}
