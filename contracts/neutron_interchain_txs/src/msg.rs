use crate::storage::{AcknowledgementResult, IntegrationTestsSudoFailureMock};
use cosmwasm_std::Uint128;
use neutron_std::types::ibc::core::channel::v1::Order;
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
        ordering: Option<Order>,
    },
    SetFees {
        fees: Option<Fees>,
    },
    Delegate {
        interchain_account_id: String,
        validator: String,
        amount: Uint128,
        denom: String,
        timeout: Option<u64>,
    },
    DelegateDoubleAck {
        interchain_account_id: String,
        validator: String,
        amount: Uint128,
        denom: String,
        timeout: Option<u64>,
    },
    Undelegate {
        interchain_account_id: String,
        validator: String,
        amount: Uint128,
        denom: String,
        timeout: Option<u64>,
    },
    CleanAckResults {},
    ResubmitFailure {
        failure_id: u64,
    },
    /// Used only in integration tests framework to simulate failures.
    /// After executing this message, any sudo call to the contract will result in an error.
    IntegrationTestsSetSudoFailureMock {
        state: IntegrationTestsSudoFailureMock,
    },
    /// Used only in integration tests framework to simulate failures.
    /// After executing this message, any sudo call to the contract will result in a submessage
    /// processing error.
    IntegrationTestsSetSudoSubmsgFailureMock {},
    /// Used only in integration tests framework to simulate failures.
    /// After executing this message, any sudo call to the contract will result in a submessage
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Fees {
    pub denom: String,
    pub recv_fee: Uint128,
    pub ack_fee: Uint128,
    pub timeout_fee: Uint128,
}
