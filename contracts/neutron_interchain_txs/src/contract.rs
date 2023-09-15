// Copyright 2022 Neutron
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
use cosmos_sdk_proto::cosmos::staking::v1beta1::{
    MsgDelegate, MsgDelegateResponse, MsgUndelegate, MsgUndelegateResponse,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Binary, Coin as CosmosCoin, CosmosMsg, CustomQuery, Deps, DepsMut, Env,
    MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, Uint128,
};
use cw2::set_contract_version;
use prost::Message;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::integration_tests_mock_handlers::{
    set_sudo_failure_mock, set_sudo_submsg_failure_in_reply_mock, set_sudo_submsg_failure_mock,
    unset_sudo_failure_mock,
};
use crate::msg::{
    AcknowledgementResultsResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
};
use neutron_sdk::bindings::msg::{IbcFee, MsgSubmitTxResponse, NeutronMsg};
use neutron_sdk::bindings::query::{NeutronQuery, QueryInterchainAccountAddressResponse};
use neutron_sdk::bindings::types::ProtobufAny;
use neutron_sdk::interchain_txs::helpers::{
    decode_acknowledgement_response, decode_message_response, get_port_id,
};
use neutron_sdk::sudo::msg::{RequestPacket, SudoMsg};
use neutron_sdk::NeutronResult;

use crate::storage::{
    add_error_to_queue, read_errors_from_queue, read_reply_payload, read_sudo_payload,
    save_reply_payload, save_sudo_payload, AcknowledgementResult, DoubleDelegateInfo,
    IntegrationTestsSudoFailureMock, IntegrationTestsSudoSubmsgFailureMock, SudoPayload,
    ACKNOWLEDGEMENT_RESULTS, IBC_FEE, INTEGRATION_TESTS_SUDO_FAILURE_MOCK,
    INTEGRATION_TESTS_SUDO_SUBMSG_FAILURE_MOCK, INTERCHAIN_ACCOUNTS, REGISTER_FEE,
    SUDO_FAILING_SUBMSG_REPLY_ID, SUDO_PAYLOAD_REPLY_ID, TEST_COUNTER_ITEM,
};

// Default timeout for SubmitTX is two weeks
const DEFAULT_TIMEOUT_SECONDS: u64 = 60 * 60 * 24 * 7 * 2;

const CONTRACT_NAME: &str = concat!("crates.io:neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
struct OpenAckVersion {
    version: String,
    controller_connection_id: String,
    host_connection_id: String,
    address: String,
    encoding: String,
    tx_type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
struct ExecuteDelegateInfo {
    pub interchain_account_id: String,
    pub validator: String,
    pub amount: u128,
    pub denom: String,
    pub timeout: Option<u64>,
    pub info: Option<DoubleDelegateInfo>,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    deps.api.debug("WASMDEBUG: instantiate");
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    REGISTER_FEE.save(deps.storage, &coins(1000, "untrn"))?;
    Ok(Response::default())
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
        ExecuteMsg::Register {
            connection_id,
            interchain_account_id,
        } => execute_register_ica(deps, env, connection_id, interchain_account_id),
        ExecuteMsg::Delegate {
            validator,
            interchain_account_id,
            amount,
            denom,
            timeout,
        } => execute_delegate(
            deps,
            env,
            ExecuteDelegateInfo {
                interchain_account_id,
                validator,
                amount,
                denom,
                timeout,
                info: None,
            },
        ),
        ExecuteMsg::DelegateDoubleAck {
            validator,
            interchain_account_id,
            amount,
            denom,
            timeout,
        } => execute_delegate_double_ack(
            deps,
            env,
            ExecuteDelegateInfo {
                interchain_account_id: interchain_account_id.clone(),
                validator: validator.clone(),
                amount,
                denom: denom.clone(),
                timeout,
                info: Some(DoubleDelegateInfo {
                    interchain_account_id,
                    validator,
                    denom,
                    amount,
                }),
            },
        ),
        ExecuteMsg::Undelegate {
            validator,
            interchain_account_id,
            amount,
            denom,
            timeout,
        } => execute_undelegate(
            deps,
            env,
            interchain_account_id,
            validator,
            amount,
            denom,
            timeout,
        ),
        ExecuteMsg::SetFees {
            denom,
            recv_fee,
            ack_fee,
            timeout_fee,
        } => execute_set_fees(deps, denom, recv_fee, ack_fee, timeout_fee),
        ExecuteMsg::CleanAckResults {} => execute_clean_ack_results(deps),
        ExecuteMsg::ResubmitFailure { failure_id } => execute_resubmit_failure(deps, failure_id),

        // The section below is used only in integration tests framework to simulate failures.
        ExecuteMsg::IntegrationTestsSetSudoFailureMock { state } => {
            set_sudo_failure_mock(deps, state)
        }
        ExecuteMsg::IntegrationTestsSetSudoSubmsgFailureMock {} => {
            set_sudo_submsg_failure_mock(deps)
        }
        ExecuteMsg::IntegrationTestsSetSudoSubmsgReplyFailureMock {} => {
            set_sudo_submsg_failure_in_reply_mock(deps)
        }
        ExecuteMsg::IntegrationTestsUnsetSudoFailureMock {} => unset_sudo_failure_mock(deps),
        ExecuteMsg::IntegrationTestsSudoSubmsg {} => integration_tests_sudo_submsg(deps),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    match msg {
        QueryMsg::InterchainAccountAddress {
            interchain_account_id,
            connection_id,
        } => query_interchain_address(deps, env, interchain_account_id, connection_id),
        QueryMsg::InterchainAccountAddressFromContract {
            interchain_account_id,
        } => query_interchain_address_contract(deps, env, interchain_account_id),
        QueryMsg::AcknowledgementResult {
            interchain_account_id,
            sequence_id,
        } => query_acknowledgement_result(deps, env, interchain_account_id, sequence_id),
        QueryMsg::AcknowledgementResults {} => query_acknowledgement_results(deps),
        QueryMsg::ErrorsQueue {} => query_errors_queue(deps),
    }
}

pub fn query_interchain_address(
    deps: Deps<NeutronQuery>,
    env: Env,
    interchain_account_id: String,
    connection_id: String,
) -> NeutronResult<Binary> {
    let query = NeutronQuery::InterchainAccountAddress {
        owner_address: env.contract.address.to_string(),
        interchain_account_id,
        connection_id,
    };

    let res: QueryInterchainAccountAddressResponse = deps.querier.query(&query.into())?;
    Ok(to_binary(&res)?)
}

pub fn query_interchain_address_contract(
    deps: Deps<NeutronQuery>,
    env: Env,
    interchain_account_id: String,
) -> NeutronResult<Binary> {
    Ok(to_binary(&get_ica(deps, &env, &interchain_account_id)?)?)
}

pub fn query_acknowledgement_result(
    deps: Deps<NeutronQuery>,
    env: Env,
    interchain_account_id: String,
    sequence_id: u64,
) -> NeutronResult<Binary> {
    let port_id = get_port_id(env.contract.address.as_str(), &interchain_account_id);
    let res = ACKNOWLEDGEMENT_RESULTS.may_load(deps.storage, (port_id, sequence_id))?;
    Ok(to_binary(&res)?)
}

pub fn query_acknowledgement_results(deps: Deps<NeutronQuery>) -> NeutronResult<Binary> {
    let results: Vec<AcknowledgementResultsResponse> = ACKNOWLEDGEMENT_RESULTS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .take(100)
        .map(|p| {
            p.map(|(key_pair, ack_result)| AcknowledgementResultsResponse {
                ack_result,
                port_id: key_pair.0,
                sequence_id: key_pair.1,
            })
        })
        .collect::<StdResult<Vec<AcknowledgementResultsResponse>>>()?;

    Ok(to_binary(&results)?)
}

pub fn query_errors_queue(deps: Deps<NeutronQuery>) -> NeutronResult<Binary> {
    let res = read_errors_from_queue(deps.storage)?;
    Ok(to_binary(&res)?)
}

fn msg_with_sudo_callback<C: Into<CosmosMsg<T>>, T>(
    deps: DepsMut,
    msg: C,
    payload: SudoPayload,
) -> StdResult<SubMsg<T>> {
    save_reply_payload(deps.storage, payload)?;
    Ok(SubMsg::reply_on_success(msg, SUDO_PAYLOAD_REPLY_ID))
}

fn execute_set_fees(
    deps: DepsMut,
    denom: String,
    recv_fee: u128,
    ack_fee: u128,
    timeout_fee: u128,
) -> StdResult<Response<NeutronMsg>> {
    let fees = IbcFee {
        recv_fee: vec![CosmosCoin {
            denom: denom.clone(),
            amount: Uint128::from(recv_fee),
        }],
        ack_fee: vec![CosmosCoin {
            denom: denom.clone(),
            amount: Uint128::from(ack_fee),
        }],
        timeout_fee: vec![CosmosCoin {
            denom,
            amount: Uint128::from(timeout_fee),
        }],
    };
    IBC_FEE.save(deps.storage, &fees)?;
    Ok(Response::default())
}

fn execute_register_ica(
    deps: DepsMut,
    env: Env,
    connection_id: String,
    interchain_account_id: String,
) -> StdResult<Response<NeutronMsg>> {
    let register_fee = REGISTER_FEE.load(deps.storage)?;
    let register = NeutronMsg::register_interchain_account(
        connection_id,
        interchain_account_id.clone(),
        register_fee,
    );
    let key = get_port_id(env.contract.address.as_str(), &interchain_account_id);
    INTERCHAIN_ACCOUNTS.save(deps.storage, key, &None)?;
    Ok(Response::new().add_message(register))
}

fn execute_delegate(
    deps: DepsMut,
    env: Env,
    info: ExecuteDelegateInfo,
) -> StdResult<Response<NeutronMsg>> {
    do_delegate(deps, env, info)
}

fn execute_undelegate(
    mut deps: DepsMut,
    env: Env,
    interchain_account_id: String,
    validator: String,
    amount: u128,
    denom: String,
    timeout: Option<u64>,
) -> StdResult<Response<NeutronMsg>> {
    let fee = IBC_FEE.load(deps.storage)?;
    let (delegator, connection_id) = get_ica(deps.as_ref(), &env, &interchain_account_id)?;
    let delegate_msg = MsgUndelegate {
        delegator_address: delegator,
        validator_address: validator,
        amount: Some(Coin {
            denom,
            amount: amount.to_string(),
        }),
    };
    let mut buf = Vec::new();
    buf.reserve(delegate_msg.encoded_len());

    if let Err(e) = delegate_msg.encode(&mut buf) {
        return Err(StdError::generic_err(format!("Encode error: {}", e)));
    }

    let any_msg = ProtobufAny {
        type_url: "/cosmos.staking.v1beta1.MsgUndelegate".to_string(),
        value: Binary::from(buf),
    };

    let cosmos_msg = NeutronMsg::submit_tx(
        connection_id,
        interchain_account_id.clone(),
        vec![any_msg],
        "".to_string(),
        timeout.unwrap_or(DEFAULT_TIMEOUT_SECONDS),
        fee,
    );

    let submsg = msg_with_sudo_callback(
        deps.branch(),
        cosmos_msg,
        SudoPayload {
            port_id: get_port_id(env.contract.address.as_str(), &interchain_account_id),
            message: "message".to_string(),
            info: None,
        },
    )?;

    Ok(Response::default().add_submessages(vec![submsg]))
}

fn execute_delegate_double_ack(
    deps: DepsMut,
    env: Env,
    info: ExecuteDelegateInfo,
) -> StdResult<Response<NeutronMsg>> {
    do_delegate(deps, env, info)
}

fn do_delegate(
    mut deps: DepsMut,
    env: Env,
    info: ExecuteDelegateInfo,
) -> StdResult<Response<NeutronMsg>> {
    let fee = IBC_FEE.load(deps.storage)?;
    let (delegator, connection_id) = get_ica(deps.as_ref(), &env, &info.interchain_account_id)?;
    let delegate_msg = MsgDelegate {
        delegator_address: delegator,
        validator_address: info.validator,
        amount: Some(Coin {
            denom: info.denom,
            amount: info.amount.to_string(),
        }),
    };
    let mut buf = Vec::new();
    buf.reserve(delegate_msg.encoded_len());

    if let Err(e) = delegate_msg.encode(&mut buf) {
        return Err(StdError::generic_err(format!("Encode error: {}", e)));
    }

    let any_msg = ProtobufAny {
        type_url: "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
        value: Binary::from(buf),
    };

    let cosmos_msg = NeutronMsg::submit_tx(
        connection_id,
        info.interchain_account_id.clone(),
        vec![any_msg],
        "".to_string(),
        info.timeout.unwrap_or(DEFAULT_TIMEOUT_SECONDS),
        fee,
    );

    // We use a submessage here because we need the process message reply to save
    // the outgoing IBC packet identifier for later.
    let submsg = msg_with_sudo_callback(
        deps.branch(),
        cosmos_msg,
        SudoPayload {
            port_id: get_port_id(env.contract.address.as_str(), &info.interchain_account_id),
            message: "message".to_string(),
            info: info.info,
        },
    )?;

    Ok(Response::default().add_submessages(vec![submsg]))
}

fn execute_clean_ack_results(deps: DepsMut) -> StdResult<Response<NeutronMsg>> {
    let keys: Vec<StdResult<(String, u64)>> = ACKNOWLEDGEMENT_RESULTS
        .keys(deps.storage, None, None, cosmwasm_std::Order::Descending)
        .collect();
    for key in keys {
        ACKNOWLEDGEMENT_RESULTS.remove(deps.storage, key?);
    }
    Ok(Response::default())
}

fn execute_resubmit_failure(_: DepsMut, failure_id: u64) -> StdResult<Response<NeutronMsg>> {
    let msg = NeutronMsg::submit_resubmit_failure(failure_id);
    Ok(Response::default().add_message(msg))
}

fn integration_tests_sudo_submsg(deps: DepsMut) -> StdResult<Response<NeutronMsg>> {
    if let Some(IntegrationTestsSudoSubmsgFailureMock::Enabled {}) =
        INTEGRATION_TESTS_SUDO_SUBMSG_FAILURE_MOCK.may_load(deps.storage)?
    {
        // Used only in integration tests framework to simulate failures.
        deps.api
            .debug("WASMDEBUG: sudo: mocked submsg failure on the handler");

        return Err(StdError::generic_err(
            "Integrations test mock submsg error".to_string(),
        ));
    }
    Ok(Response::default())
}

// Err result returned from the `sudo()` handler will result in the `Failure` object stored in the chain state.
// It can be resubmitted later using `NeutronMsg::ResubmitFailure { failure_id }` message.
#[allow(unreachable_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(mut deps: DepsMut, env: Env, msg: SudoMsg) -> StdResult<Response<NeutronMsg>> {
    let api = deps.api;
    api.debug(format!("WASMDEBUG: sudo: received sudo msg: {:?}", msg).as_str());

    let mock_res = INTEGRATION_TESTS_SUDO_FAILURE_MOCK.may_load(deps.storage)?;

    let failure_submsg_mock_enabled = {
        let m = INTEGRATION_TESTS_SUDO_SUBMSG_FAILURE_MOCK.may_load(deps.storage)?;
        m == Some(IntegrationTestsSudoSubmsgFailureMock::Enabled {})
            || m == Some(IntegrationTestsSudoSubmsgFailureMock::EnabledInReply {})
    };

    let mut resp: Response<NeutronMsg> = match msg.clone() {
        SudoMsg::Response { request, data } => {
            sudo_response(deps.branch(), env.clone(), request, data)?
        }
        SudoMsg::Error { request, details } => sudo_error(deps.branch(), request, details)?,
        SudoMsg::Timeout { request } => sudo_timeout(deps.branch(), env.clone(), request)?,
        SudoMsg::OpenAck {
            port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_version,
        } => sudo_open_ack(
            deps.branch(),
            env.clone(),
            port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_version,
        )?,
        _ => Response::default(),
    };

    match mock_res {
        Some(IntegrationTestsSudoFailureMock::Enabled) => {
            // Used only in integration tests framework to simulate failures.
            api.debug("WASMDEBUG: sudo: mocked failure on the handler");

            return Err(StdError::generic_err(
                "Integrations test mock error".to_string(),
            ));
        }
        Some(IntegrationTestsSudoFailureMock::EnabledInfiniteLoop) => {
            // Used only in integration tests framework to simulate failures.
            api.debug("WASMDEBUG: sudo: mocked failure on the handler");

            if let SudoMsg::Response { request, data: _ } = msg {
                deps.api.debug(
                    format!(
                        "WASMDEBUG: infinite loop failure response; sequence_id = {:?}",
                        &request.sequence.unwrap_or_default().to_string()
                    )
                    .as_str(),
                );
            }

            let mut counter: u64 = 0;
            loop {
                counter = counter.checked_add(1).unwrap_or_default();
                TEST_COUNTER_ITEM.save(deps.storage, &counter)?;
            }
            TEST_COUNTER_ITEM.save(deps.storage, &counter)?;

            return Ok(Response::default());
        }
        _ => {}
    }

    if failure_submsg_mock_enabled {
        resp = resp.add_submessage(SubMsg {
            id: SUDO_FAILING_SUBMSG_REPLY_ID,
            msg: CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::IntegrationTestsSudoSubmsg {})?,
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Success,
        })
    };

    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}

fn sudo_open_ack(
    deps: DepsMut,
    _env: Env,
    port_id: String,
    _channel_id: String,
    _counterparty_channel_id: String,
    counterparty_version: String,
) -> StdResult<Response<NeutronMsg>> {
    let parsed_version: Result<OpenAckVersion, _> =
        serde_json_wasm::from_str(counterparty_version.as_str());
    if let Ok(parsed_version) = parsed_version {
        INTERCHAIN_ACCOUNTS.save(
            deps.storage,
            port_id,
            &Some((
                parsed_version.address,
                parsed_version.controller_connection_id,
            )),
        )?;
        return Ok(Response::default());
    }
    Err(StdError::generic_err("Can't parse counterparty_version"))
}

fn sudo_response(
    mut deps: DepsMut,
    env: Env,
    request: RequestPacket,
    data: Binary,
) -> StdResult<Response<NeutronMsg>> {
    deps.api.debug(
        format!(
            "WASMDEBUG: sudo_response: sudo received: {:?} {:?}",
            request, data
        )
        .as_str(),
    );

    let seq_id = request
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;

    let channel_id = request
        .source_channel
        .ok_or_else(|| StdError::generic_err("channel_id not found"))?;

    let payload = read_sudo_payload(deps.storage, channel_id, seq_id).ok();
    if payload.is_none() {
        let error_msg = "WASMDEBUG: Error: Unable to read sudo payload";
        deps.api.debug(error_msg);
        add_error_to_queue(deps.storage, error_msg.to_string());
        return Ok(Response::default());
    }

    deps.api
        .debug(format!("WASMDEBUG: sudo_response: sudo payload: {:?}", payload).as_str());

    let parsed_data = decode_acknowledgement_response(data)?;

    let mut item_types = vec![];
    for item in parsed_data {
        let item_type = item.msg_type.as_str();
        item_types.push(item_type.to_string());
        match item_type {
            "/cosmos.staking.v1beta1.MsgUndelegate" => {
                let out: MsgUndelegateResponse = decode_message_response(&item.data)?;

                let completion_time = out.completion_time.or_else(|| {
                    let error_msg = "WASMDEBUG: sudo_response: Recoverable error. Failed to get completion time";
                    deps.api
                        .debug(error_msg);
                    add_error_to_queue(deps.storage, error_msg.to_string());
                    Some(prost_types::Timestamp::default())
                });
                deps.api
                    .debug(format!("Undelegation completion time: {:?}", completion_time).as_str());
            }
            "/cosmos.staking.v1beta1.MsgDelegate" => {
                let _out: MsgDelegateResponse = decode_message_response(&item.data)?;
            }
            _ => {
                deps.api.debug(
                    format!(
                        "This type of acknowledgement is not implemented: {:?}",
                        payload
                    )
                    .as_str(),
                );
            }
        }
    }

    if let Some(payload) = payload {
        // update but also check that we don't update same seq_id twice
        ACKNOWLEDGEMENT_RESULTS.update(
            deps.storage,
            (payload.clone().port_id, seq_id),
            |maybe_ack| -> StdResult<AcknowledgementResult> {
                match maybe_ack {
                    Some(_ack) => Err(StdError::generic_err("trying to update same seq_id")),
                    None => Ok(AcknowledgementResult::Success(item_types)),
                }
            },
        )?;

        deps.api
            .debug(format!("WASMDEBUG: payload received: {:?}", payload).as_str());

        if let Some(info) = payload.info {
            let res = {
                do_delegate(
                    deps.branch(),
                    env,
                    ExecuteDelegateInfo {
                        interchain_account_id: info.interchain_account_id,
                        validator: info.validator,
                        amount: info.amount,
                        denom: info.denom,
                        timeout: None,
                        info: None,
                    },
                )
            };

            if let Err(err) = res {
                deps.api.debug(
                    format!(
                        "WASMDEBUG: error constructing delegate from sudo: {:?}",
                        err.to_string()
                    )
                    .as_str(),
                );
            } else {
                return res;
            }
        }
    }

    Ok(Response::default())
}

fn sudo_timeout(
    deps: DepsMut,
    _env: Env,
    request: RequestPacket,
) -> StdResult<Response<NeutronMsg>> {
    deps.api
        .debug(format!("WASMDEBUG: sudo timeout request: {:?}", request).as_str());

    let seq_id = request
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;

    let channel_id = request
        .source_channel
        .ok_or_else(|| StdError::generic_err("channel_id not found"))?;

    // update but also check that we don't update same seq_id twice
    let payload = read_sudo_payload(deps.storage, channel_id, seq_id).ok();
    if let Some(payload) = payload {
        // update but also check that we don't update same seq_id twice
        ACKNOWLEDGEMENT_RESULTS.update(
            deps.storage,
            (payload.port_id, seq_id),
            |maybe_ack| -> StdResult<AcknowledgementResult> {
                match maybe_ack {
                    Some(_ack) => Err(StdError::generic_err("trying to update same seq_id")),
                    None => Ok(AcknowledgementResult::Timeout(payload.message)),
                }
            },
        )?;
    } else {
        let error_msg = "WASMDEBUG: Error: Unable to read sudo payload";
        deps.api.debug(error_msg);
        add_error_to_queue(deps.storage, error_msg.to_string());
    }

    Ok(Response::default())
}

fn sudo_error(
    deps: DepsMut,
    request: RequestPacket,
    details: String,
) -> StdResult<Response<NeutronMsg>> {
    deps.api
        .debug(format!("WASMDEBUG: sudo error: {}", details).as_str());
    deps.api
        .debug(format!("WASMDEBUG: request packet: {:?}", request).as_str());

    let seq_id = request
        .sequence
        .ok_or_else(|| StdError::generic_err("sequence not found"))?;

    let channel_id = request
        .source_channel
        .ok_or_else(|| StdError::generic_err("channel_id not found"))?;
    let payload = read_sudo_payload(deps.storage, channel_id, seq_id).ok();

    if let Some(payload) = payload {
        // update but also check that we don't update same seq_id twice
        ACKNOWLEDGEMENT_RESULTS.update(
            deps.storage,
            (payload.port_id, seq_id),
            |maybe_ack| -> StdResult<AcknowledgementResult> {
                match maybe_ack {
                    Some(_ack) => Err(StdError::generic_err("trying to update same seq_id")),
                    None => Ok(AcknowledgementResult::Error((payload.message, details))),
                }
            },
        )?;
    } else {
        let error_msg = "WASMDEBUG: Error: Unable to read sudo payload";
        deps.api.debug(error_msg);
        add_error_to_queue(deps.storage, error_msg.to_string());
    }

    Ok(Response::default())
}

fn prepare_sudo_payload(mut deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    let payload = read_reply_payload(deps.storage)?;
    let resp: MsgSubmitTxResponse = serde_json_wasm::from_slice(
        msg.result
            .into_result()
            .map_err(StdError::generic_err)?
            .data
            .ok_or_else(|| StdError::generic_err("no result"))?
            .as_slice(),
    )
    .map_err(|e| StdError::generic_err(format!("failed to parse response: {:?}", e)))?;
    deps.api
        .debug(format!("WASMDEBUG: reply msg: {:?}", resp).as_str());
    let seq_id = resp.sequence_id;
    let channel_id = resp.channel;
    save_sudo_payload(deps.branch().storage, channel_id, seq_id, payload)?;
    Ok(Response::new())
}

fn get_ica(
    deps: Deps<impl CustomQuery>,
    env: &Env,
    interchain_account_id: &str,
) -> Result<(String, String), StdError> {
    let key = get_port_id(env.contract.address.as_str(), interchain_account_id);

    INTERCHAIN_ACCOUNTS
        .load(deps.storage, key)?
        .ok_or_else(|| StdError::generic_err("Interchain account is not created yet"))
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    deps.api
        .debug(format!("WASMDEBUG: reply msg: {:?}", msg).as_str());
    match msg.id {
        SUDO_PAYLOAD_REPLY_ID => prepare_sudo_payload(deps, env, msg),
        SUDO_FAILING_SUBMSG_REPLY_ID => {
            if let Some(IntegrationTestsSudoSubmsgFailureMock::EnabledInReply {}) =
                INTEGRATION_TESTS_SUDO_SUBMSG_FAILURE_MOCK.may_load(deps.storage)?
            {
                // Used only in integration tests framework to simulate failures.
                deps.api
                    .debug("WASMDEBUG: sudo: mocked reply failure on the handler");

                return Err(StdError::GenericErr {
                    msg: "Integrations test mock reply error".to_string(),
                });
            }
            Ok(Response::default())
        }
        _ => Err(StdError::generic_err(format!(
            "unsupported reply message id {}",
            msg.id
        ))),
    }
}
