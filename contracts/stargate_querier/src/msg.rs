use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    QueryBankBalance {
        address: String,
        denom: String,
    },
    QueryBankDenomMetadata {
        denom: String,
    },
    QueryBankParams {},
    QueryBankSupplyOf {
        denom: String,
    },
    QueryAuthAccount {
        address: String,
    },
    QueryTransferDenomTrace {
        hash: String,
    },
    QueryIbcClientState {
        client_id: String,
    },
    QueryIbcConsensusState {
        client_id: String,
        revision_number: u64,
        revision_height: u64,
        latest_height: bool,
    },
    QueryIbcConnection {
        connection_id: String,
    },
    TokenfactoryParams {},
    TokenfactoryDenomAuthorityMetadata {
        denom: String,
    },
    TokenfactoryDenomsFromCreator {
        creator: String,
    },
    ContractmanagerAddressFailures {
        address: String,
    },
    ContractmanagerFailures {
        address: String,
    },
    QueryInterchaintxParams {},
    QueryInterchainqueriesParams {},
    QueryFeeburnerParams {},
}
