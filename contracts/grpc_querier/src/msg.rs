use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ExecuteMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    BankBalance {
        address: String,
        denom: String,
    },
    BankDenomMetadata {
        denom: String,
    },
    BankParams {},
    BankSupplyOf {
        denom: String,
    },
    AuthAccount {
        address: String,
    },
    TransferDenomTrace {
        hash: String,
    },
    TransferEscrowAddress {
        port_id: String,
        channel_id: String,
    },
    IbcClientState {
        client_id: String,
    },
    IbcConsensusState {
        client_id: String,
        revision_number: u64,
        revision_height: u64,
        latest_height: bool,
    },
    IbcConnection {
        connection_id: String,
    },
    TokenfactoryParams {},
    TokenfactoryDenomAuthorityMetadata {
        creator: String,
        subdenom: String,
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
    InterchaintxsParams {},
    InterchainqueriesParams {},
    FeeburnerParams {},
    FeeburnerTotalBurnedNeutronsAmount {},
}
