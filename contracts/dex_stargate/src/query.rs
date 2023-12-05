use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InterchainQueries {
    InterchainQueryResult {
        query_id: u64,
    },
    InterchainAccountAddress {
        owner_address: String,
        interchain_account_id: String,
        connection_id: String,
    },
    RegisteredInterchainQueries {},
    RegisteredInterchainQuery {
        query_id: u64,
    },
    MinIbcFee {},
    FullDenom {
        creator_addr: String,
        subdenom: String,
    },
    DenomAdmin {
        subdenom: String,
    },
}
