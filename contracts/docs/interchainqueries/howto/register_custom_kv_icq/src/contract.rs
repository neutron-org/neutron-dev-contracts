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

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{ICQ_ID_TO_WATCHED_ADDR, REMOTE_ACCOUNTS};
use crate::types::BaseAccount;
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply,
    ReplyOn, Response, StdError, StdResult, SubMsg,
};
use cw2::set_contract_version;
use neutron_sdk::{
    interchain_queries::{helpers::decode_and_convert, queries::query_kv_result, types::QueryType},
    sudo::msg::SudoMsg,
    {NeutronError, NeutronResult},
};
use neutron_std::types::neutron::interchainqueries::{
    KvKey, MsgRegisterInterchainQuery, MsgRegisterInterchainQueryResponse,
};
use prost::Message;

const CONTRACT_NAME: &str = concat!("crates.io:neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Reply ID used to tell this kind of reply call apart.
const REGISTER_ACCOUNT_ICQ_REPLY_ID: u64 = 1;

/// Store key for standard **auth** Cosmos-SDK module
pub const AUTH_STORE_KEY: &str = "acc";
/// Storage prefix for account-by-address store
/// <https://github.com/cosmos/cosmos-sdk/blob/853dbbf3e84900214137805d78e325ecd56fd68f/x/auth/types/keys.go#L22-L23>
pub const ACCOUNTS_PREFIX: u8 = 0x01;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> NeutronResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> NeutronResult<Response> {
    match msg {
        ExecuteMsg::RegisterAccountQuery {
            connection_id,
            addr,
            update_period,
        } => register_account_query(env, connection_id, addr, update_period),
    }
}

pub fn register_account_query(
    env: Env,
    connection_id: String,
    addr: String,
    update_period: u64,
) -> NeutronResult<Response> {
    // compose key as accounts store prefix + hex address representation
    let mut key: Vec<u8> = vec![ACCOUNTS_PREFIX];
    key.extend_from_slice(decode_and_convert(&addr)?.as_slice());

    let msg = MsgRegisterInterchainQuery {
        query_type: QueryType::KV.into(),
        keys: vec![KvKey {
            path: AUTH_STORE_KEY.to_string(),
            key,
        }],
        transactions_filter: String::default(),
        connection_id,
        update_period,
        sender: env.contract.address.to_string(),
    };

    // Send the ICQ registration message as a submessage to receive a reply callback
    Ok(Response::new().add_submessage(SubMsg {
        id: REGISTER_ACCOUNT_ICQ_REPLY_ID,
        payload: to_json_binary(&addr)?,
        msg: msg.into(),
        gas_limit: None,
        reply_on: ReplyOn::Success,
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    match msg {
        QueryMsg::Account { address } => query_account(deps, address),
    }
}

pub fn query_account(deps: Deps, addr: String) -> NeutronResult<Binary> {
    Ok(to_json_binary(
        &REMOTE_ACCOUNTS
            .may_load(deps.storage, addr)
            .unwrap_or_default(),
    )?)
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> NeutronResult<Response> {
    match msg.id {
        REGISTER_ACCOUNT_ICQ_REPLY_ID => {
            // decode the reply msg result as MsgRegisterInterchainQueryResponse
            let resp = MsgRegisterInterchainQueryResponse::decode(
                msg.result
                    .into_result()
                    .map_err(StdError::generic_err)?
                    .msg_responses[0]
                    .clone()
                    .value
                    .to_vec()
                    .as_slice(),
            )?;

            // memorize the address that corresponds to the query id to use it later in the
            // SudoMsg::KVQueryResult handler.
            let addr: String = from_json(&msg.payload)?;
            ICQ_ID_TO_WATCHED_ADDR.save(deps.storage, resp.id, &addr)?;

            Ok(Response::default())
        }
        _ => Err(NeutronError::InvalidReplyID(msg.id)),
    }
}

#[entry_point]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> NeutronResult<Response> {
    match msg {
        SudoMsg::KVQueryResult { query_id } => sudo_kv_query_result(deps, query_id),
        _ => Ok(Response::default()),
    }
}

/// The contract's callback for KV query results. Note that only the query id is provided, so you
/// need to read the query result from the state.
pub fn sudo_kv_query_result(deps: DepsMut, query_id: u64) -> NeutronResult<Response> {
    // Get the last submitted ICQ result from the Neutron ICQ module storage and decode it
    // as BaseAccount using its KVReconstruct::reconstruct implementation.
    let account_resp: BaseAccount = query_kv_result(deps.as_ref(), query_id)?;
    // Get the address that was registered for the ICQ
    let addr = ICQ_ID_TO_WATCHED_ADDR.load(deps.storage, query_id)?;

    // Put your business logic here
    // For this example we just preserve the freshly fetched account in the contract's state
    REMOTE_ACCOUNTS.save(deps.storage, addr, &account_resp)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
