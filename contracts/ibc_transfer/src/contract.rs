use cosmwasm_std::{
    coin, entry_point, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdError, StdResult, SubMsg, Uint128,
};
use cw2::set_contract_version;
use neutron_sdk::interchain_txs::helpers::decode_message_response;
use neutron_sdk::{
    bindings::msg::{IbcFee, NeutronMsg},
    sudo::msg::{RequestPacket, RequestPacketTimeoutHeight, TransferSudoMsg},
};
use neutron_std::types::neutron::transfer::MsgTransferResponse;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{
    read_reply_payload, read_sudo_payload, save_reply_payload, save_sudo_payload, IBC_FEE,
    IBC_SUDO_ID_RANGE_END, IBC_SUDO_ID_RANGE_START, TEST_COUNTER_ITEM,
};

use crate::{
    integration_tests_mock_handlers::{set_sudo_failure_mock, unset_sudo_failure_mock},
    state::{IntegrationTestsSudoFailureMock, INTEGRATION_TESTS_SUDO_FAILURE_MOCK},
};

// Default timeout for IbcTransfer is 10000000 blocks
const DEFAULT_TIMEOUT_HEIGHT: u64 = 10000000;

const CONTRACT_NAME: &str = concat!("crates.io:neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: instantiate");
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Send {
        channel: String,
        to: String,
        denom: String,
        amount: Uint128,
        timeout_height: Option<u64>,
    },
    SetFees {
        recv_fee: Uint128,
        ack_fee: Uint128,
        timeout_fee: Uint128,
        denom: String,
    },
    ResubmitFailure {
        failure_id: u64,
    },
    /// Used only in integration tests framework to simulate failures.
    /// After executing this message, contract will fail, all of this happening
    /// in sudo callback handler.
    IntegrationTestsSetSudoFailureMock {
        state: IntegrationTestsSudoFailureMock,
    },
    /// Used only in integration tests framework to simulate failures.
    /// After executing this message, contract will revert back to normal behaviour.
    IntegrationTestsUnsetSudoFailureMock {},
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response<NeutronMsg>> {
    deps.api
        .debug(format!("WASMDEBUG: execute: received msg: {:?}", msg).as_str());
    match msg {
        // NOTE: this is an example contract that shows how to make IBC transfers!
        // Please add necessary authorization or other protection mechanisms
        // if you intend to send funds over IBC
        ExecuteMsg::Send {
            channel,
            to,
            denom,
            amount,
            timeout_height,
        } => execute_send(deps, env, channel, to, denom, amount, timeout_height),

        ExecuteMsg::SetFees {
            recv_fee,
            ack_fee,
            timeout_fee,
            denom,
        } => execute_set_fees(deps, recv_fee, ack_fee, timeout_fee, denom),

        ExecuteMsg::ResubmitFailure { failure_id } => execute_resubmit_failure(deps, failure_id),

        // Used only in integration tests framework to simulate failures.
        // After executing this message, contract fail, all of this happening
        // in sudo callback handler.
        ExecuteMsg::IntegrationTestsSetSudoFailureMock { state } => {
            set_sudo_failure_mock(deps, state)
        }
        ExecuteMsg::IntegrationTestsUnsetSudoFailureMock {} => unset_sudo_failure_mock(deps),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Type1 {
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Type2 {
    pub data: String,
}

fn sudo_callback1(deps: Deps, payload: Type1) -> StdResult<Response> {
    deps.api
        .debug(format!("WASMDEBUG: callback1: sudo payload: {:?}", payload).as_str());
    Ok(Response::new())
}

fn sudo_callback2(deps: Deps, payload: Type2) -> StdResult<Response> {
    deps.api
        .debug(format!("WASMDEBUG: callback2: sudo payload: {:?}", payload).as_str());
    Ok(Response::new())
}

#[derive(Serialize, Deserialize)]
pub enum SudoPayload {
    HandlerPayload1(Type1),
    HandlerPayload2(Type2),
}

fn msg_with_sudo_callback<C: Into<CosmosMsg<T>>, T>(
    deps: DepsMut,
    msg: C,
    payload: SudoPayload,
) -> StdResult<SubMsg<T>> {
    let id = save_reply_payload(deps.storage, payload)?;
    Ok(SubMsg::reply_on_success(msg, id))
}

fn prepare_sudo_payload(mut deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    let payload = read_reply_payload(deps.storage, msg.id)?;
    let resp: MsgTransferResponse = decode_message_response(
        &msg.result
            .into_result()
            .map_err(StdError::generic_err)?
            .msg_responses[0] // msg_responses must have exactly one Msg response: https://github.com/neutron-org/neutron/blob/28b1d2ce968aaf1866e92d5286487f079eba3370/wasmbinding/message_plugin.go#L307
            .clone()
            .value
            .to_vec(),
    )
    .map_err(|e| StdError::generic_err(format!("failed to parse response: {:?}", e)))?;
    let seq_id = resp.sequence_id;
    let channel_id = resp.channel;
    save_sudo_payload(deps.branch().storage, channel_id, seq_id, payload)?;
    Ok(Response::new())
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        IBC_SUDO_ID_RANGE_START..=IBC_SUDO_ID_RANGE_END => prepare_sudo_payload(deps, env, msg),
        _ => Err(StdError::generic_err(format!(
            "unsupported reply message id {}",
            msg.id
        ))),
    }
}

fn get_fee_item(denom: String, amount: Uint128) -> Vec<Coin> {
    if amount == Uint128::new(0) {
        vec![]
    } else {
        vec![coin(amount.u128(), denom)]
    }
}

fn execute_set_fees(
    deps: DepsMut,
    recv_fee: Uint128,
    ack_fee: Uint128,
    timeout_fee: Uint128,
    denom: String,
) -> StdResult<Response<NeutronMsg>> {
    let fee = IbcFee {
        recv_fee: get_fee_item(denom.clone(), recv_fee),
        ack_fee: get_fee_item(denom.clone(), ack_fee),
        timeout_fee: get_fee_item(denom, timeout_fee),
    };

    IBC_FEE.save(deps.storage, &fee)?;

    Ok(Response::default())
}

fn execute_send(
    mut deps: DepsMut,
    env: Env,
    channel: String,
    to: String,
    denom: String,
    amount: Uint128,
    timeout_height: Option<u64>,
) -> StdResult<Response<NeutronMsg>> {
    let fee = IBC_FEE.load(deps.storage)?;
    let coin1 = coin(amount.u128(), denom.clone());
    let msg1 = NeutronMsg::IbcTransfer {
        source_port: "transfer".to_string(),
        source_channel: channel.clone(),
        sender: env.contract.address.to_string(),
        receiver: to.clone(),
        token: coin1,
        timeout_height: RequestPacketTimeoutHeight {
            revision_number: Some(2),
            revision_height: timeout_height.or(Some(DEFAULT_TIMEOUT_HEIGHT)),
        },
        timeout_timestamp: 0,
        fee: fee.clone(),
        memo: "".to_string(),
    };
    let coin2 = coin(2 * amount.u128(), denom);
    let msg2 = NeutronMsg::IbcTransfer {
        source_port: "transfer".to_string(),
        source_channel: channel,
        sender: env.contract.address.to_string(),
        receiver: to,
        token: coin2,
        timeout_height: RequestPacketTimeoutHeight {
            revision_number: Some(2),
            revision_height: timeout_height.or(Some(DEFAULT_TIMEOUT_HEIGHT)),
        },
        timeout_timestamp: 0,
        fee,
        memo: "".to_string(),
    };
    let submsg1 = msg_with_sudo_callback(
        deps.branch(),
        msg1,
        SudoPayload::HandlerPayload1(Type1 {
            message: "message".to_string(),
        }),
    )?;
    let submsg2 = msg_with_sudo_callback(
        deps.branch(),
        msg2,
        SudoPayload::HandlerPayload2(Type2 {
            data: "data".to_string(),
        }),
    )?;
    deps.as_ref()
        .api
        .debug(format!("WASMDEBUG: execute_send: sent submsg1: {:?}", submsg1).as_str());
    deps.api
        .debug(format!("WASMDEBUG: execute_send: sent submsg2: {:?}", submsg2).as_str());

    Ok(Response::default().add_submessages(vec![submsg1, submsg2]))
}

fn execute_resubmit_failure(_: DepsMut, failure_id: u64) -> StdResult<Response<NeutronMsg>> {
    let msg = NeutronMsg::submit_resubmit_failure(failure_id);
    Ok(Response::default().add_message(msg))
}

// Err result returned from the `sudo()` handler will result in the `Failure` object stored in the chain state.
// It can be resubmitted later using `NeutronMsg::ResubmitFailure { failure_id }` message.
#[allow(unreachable_code)]
#[entry_point]
pub fn sudo(deps: DepsMut, _env: Env, msg: TransferSudoMsg) -> StdResult<Response> {
    match INTEGRATION_TESTS_SUDO_FAILURE_MOCK.may_load(deps.storage)? {
        Some(IntegrationTestsSudoFailureMock::Enabled) => {
            // Used only in integration tests framework to simulate failures.
            deps.api
                .debug("WASMDEBUG: sudo: mocked failure on the handler");

            return Err(StdError::generic_err(
                "Integrations test mock error".to_string(),
            ));
        }
        Some(IntegrationTestsSudoFailureMock::EnabledInfiniteLoop) => {
            // Used only in integration tests framework to simulate failures.
            deps.api
                .debug("WASMDEBUG: sudo: mocked infinite loop failure on the handler");

            if let TransferSudoMsg::Response { request, data: _ } = msg {
                deps.api.debug(
                    format!(
                        "WASMDEBUG: infinite loop failure response; sequence_id = {:?}",
                        &request.sequence.unwrap_or_default().to_string()
                    )
                    .as_str(),
                );
            }

            let mut counter: u64 = 0;
            while counter < 18_446_744_073_709_551_615u64 {
                counter = counter.checked_add(1).unwrap_or_default();
                TEST_COUNTER_ITEM.save(deps.storage, &counter)?;
            }
            deps.api.debug("WASMDEBUG: after infinite loop");
            TEST_COUNTER_ITEM.save(deps.storage, &counter)?;

            return Ok(Response::default());
        }
        _ => {}
    }

    match msg {
        TransferSudoMsg::Response { request, data } => sudo_response(deps, request, data),
        TransferSudoMsg::Error { request, details } => sudo_error(deps, request, details),
        TransferSudoMsg::Timeout { request } => sudo_timeout(deps, request),
    }
}

fn sudo_error(deps: DepsMut, req: RequestPacket, data: String) -> StdResult<Response> {
    deps.api.debug(
        format!(
            "WASMDEBUG: sudo_error: sudo error received: {:?} {}",
            req, data
        )
        .as_str(),
    );
    Ok(Response::new())
}

fn sudo_timeout(deps: DepsMut, req: RequestPacket) -> StdResult<Response> {
    deps.api.debug(
        format!(
            "WASMDEBUG: sudo_timeout: sudo timeout ack received: {:?}",
            req
        )
        .as_str(),
    );
    Ok(Response::new())
}

fn sudo_response(deps: DepsMut, req: RequestPacket, data: Binary) -> StdResult<Response> {
    deps.api.debug(
        format!(
            "WASMDEBUG: sudo_response: sudo received: {:?} {}",
            req, data
        )
        .as_str(),
    );
    let seq_id = req
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;
    let channel_id = req
        .source_channel
        .ok_or_else(|| StdError::generic_err("channel_id not found"))?;
    match read_sudo_payload(deps.storage, channel_id, seq_id)? {
        SudoPayload::HandlerPayload1(t1) => sudo_callback1(deps.as_ref(), t1),
        SudoPayload::HandlerPayload2(t2) => sudo_callback2(deps.as_ref(), t2),
    }
    // at this place we can safely remove the data under (channel_id, seq_id) key
    // but it costs an extra gas, so its on you how to use the storage
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
