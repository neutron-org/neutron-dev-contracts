use crate::storage::AcknowledgementResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// this query goes to neutron and get stored ICA with a specific query
    InterchainAccountAddress {
        interchain_account_id: String,
        connection_id: String,
    },
    // this query returns ICA from contract store, which saved from acknowledgement
    InterchainAccountAddressFromContract {
        interchain_account_id: String,
    },
    // this query returns acknowledgement result after interchain transaction
    AcknowledgementResult {
        interchain_account_id: String,
        sequence_id: u64,
    },
    // this query returns all acknowledgements stored in the contract's state
    AcknowledgementResults {},
    // this query returns non-critical errors list
    ErrorsQueue {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Register {
        connection_id: String,
        interchain_account_id: String,
    },
    SetFees {
        denom: String,
        recv_fee: u128,
        ack_fee: u128,
        timeout_fee: u128,
    },
    Delegate {
        interchain_account_id: String,
        validator: String,
        amount: u128,
        denom: String,
        timeout: Option<u64>,
    },
    Undelegate {
        interchain_account_id: String,
        validator: String,
        amount: u128,
        denom: String,
        timeout: Option<u64>,
    },
    CleanAckResults {},
    /// Used only in integration tests framework to simulate failures.
    /// After executing this message, any sudo call to the contract will result in an error.
    IntegrationTestsSetSudoFailureMock {},
    /// Used only in integration tests framework to simulate failures.
    /// After executing this message, any sudo call to the contract will result in an submessage
    /// processing error.
    IntegrationTestsSetSudoSubmsgFailureMock {},
    /// Used only in integration tests framework to simulate failures.
    /// After executing this message, any sudo call to the contract will result in an submessage
    /// reply processing error.
    IntegrationTestsSetSudoSubmsgReplyFailureMock {},
    /// Used only in integration tests framework to simulate failures.
    /// After executing this message, contract will revert back to normal behaviour.
    IntegrationTestsUnsetSudoFailureMock {},
    /// Used only in integration tests framework to simulate failures.
    /// If the IntegrationTestsSetSudoSubmsgFailureMock has been called, this message will fail.
    IntegrationTestsSudoSubmsg {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AcknowledgementResultsResponse {
    pub ack_result: AcknowledgementResult,
    pub port_id: String,
    pub sequence_id: u64,
}
