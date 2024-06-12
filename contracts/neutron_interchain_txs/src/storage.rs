use cosmwasm_std::Uint128;
use cosmwasm_std::{from_json, to_json_vec, Binary, Coin, Order, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use neutron_sdk::bindings::msg::IbcFee;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SudoPayload {
    pub message: String,
    pub port_id: String,
    pub info: Option<DoubleDelegateInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DoubleDelegateInfo {
    pub interchain_account_id: String,
    pub validator: String,
    pub denom: String,
    pub amount: Uint128,
}

pub const SUDO_PAYLOAD_REPLY_ID: u64 = 1;
pub const SUDO_FAILING_SUBMSG_REPLY_ID: u64 = 2;
// only used to make sure `sudo()` handler gets OpenAck message with correct port_id and channel_id
pub const REGISTER_ICA_REPLY_ID: u64 = 3;

pub const IBC_FEE: Item<IbcFee> = Item::new("ibc_fee");
pub const REGISTER_FEE: Item<Vec<Coin>> = Item::new("register_fee");
pub const REPLY_ID_STORAGE: Item<Vec<u8>> = Item::new("reply_queue_id");
pub const SUDO_PAYLOAD: Map<(String, u64), Vec<u8>> = Map::new("sudo_payload");
pub const INTERCHAIN_ACCOUNTS: Map<String, Option<(String, String)>> =
    Map::new("interchain_accounts");
// only used to make sure `sudo()` handler gets OpenAck message with correct port_id and channel_id
pub const ICA_CHANNELS: Map<String, String> = Map::new("ica_channels");

// interchain transaction responses - ack/err/timeout state to query later
pub const ACKNOWLEDGEMENT_RESULTS: Map<(String, u64), AcknowledgementResult> =
    Map::new("acknowledgement_results");

pub const ERRORS_QUEUE: Map<u32, String> = Map::new("errors_queue");

/// Serves for storing acknowledgement calls for interchain transactions
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AcknowledgementResult {
    /// Success - Got success acknowledgement in sudo with array of message item types in it
    Success(Vec<String>),
    /// Error - Got error acknowledgement in sudo with payload message in it and error details
    Error((String, String)),
    /// Timeout - Got timeout acknowledgement in sudo with payload message in it
    Timeout(String),
}

pub fn save_reply_payload(store: &mut dyn Storage, payload: SudoPayload) -> StdResult<()> {
    REPLY_ID_STORAGE.save(store, &to_json_vec(&payload)?)
}

pub fn read_reply_payload(store: &dyn Storage) -> StdResult<SudoPayload> {
    let data = REPLY_ID_STORAGE.load(store)?;
    from_json(Binary::new(data))
}

pub fn add_error_to_queue(store: &mut dyn Storage, error_msg: String) -> Option<()> {
    let result = ERRORS_QUEUE
        .keys(store, None, None, Order::Descending)
        .next()
        .and_then(|data| data.ok())
        .map(|c| c + 1)
        .or(Some(0));

    result.and_then(|idx| ERRORS_QUEUE.save(store, idx, &error_msg).ok())
}

pub fn read_errors_from_queue(store: &dyn Storage) -> StdResult<Vec<(Vec<u8>, String)>> {
    ERRORS_QUEUE
        .range_raw(store, None, None, Order::Ascending)
        .collect()
}

pub fn read_sudo_payload(
    store: &dyn Storage,
    channel_id: String,
    seq_id: u64,
) -> StdResult<SudoPayload> {
    let data = SUDO_PAYLOAD.load(store, (channel_id, seq_id))?;
    from_json(Binary::new(data))
}

pub fn save_sudo_payload(
    store: &mut dyn Storage,
    channel_id: String,
    seq_id: u64,
    payload: SudoPayload,
) -> StdResult<()> {
    SUDO_PAYLOAD.save(store, (channel_id, seq_id), &to_json_vec(&payload)?)
}

/// Used only in integration tests framework to simulate failures in sudo handler.
pub const INTEGRATION_TESTS_SUDO_FAILURE_MOCK: Item<IntegrationTestsSudoFailureMock> =
    Item::new("integration_tests_sudo_failure_mock");
/// Used only in integration tests framework to simulate failures in submessages created in
/// sudo handler.
pub const INTEGRATION_TESTS_SUDO_SUBMSG_FAILURE_MOCK: Item<IntegrationTestsSudoSubmsgFailureMock> =
    Item::new("integration_tests_sudo_submsg_failure_mock");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationTestsSudoFailureMock {
    Enabled,
    EnabledInfiniteLoop,
    Disabled,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationTestsSudoSubmsgFailureMock {
    Enabled,
    EnabledInReply,
    Disabled,
}

// just to do something in infinite loop
pub const TEST_COUNTER_ITEM: Item<u64> = Item::new("test_counter");
