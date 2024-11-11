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
use crate::state::{ICQ_ID_TO_WATCHED_ADDR, REMOTE_BALANCES};
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply,
    ReplyOn, Response, StdError, StdResult, SubMsg,
};
use cw2::set_contract_version;
use neutron_sdk::interchain_queries::v047::{
    queries::query_balance, register_queries::new_register_balances_query_msg,
};
use neutron_sdk::sudo::msg::SudoMsg;
use neutron_sdk::{NeutronError, NeutronResult};
use neutron_std::types::neutron::interchainqueries::MsgRegisterInterchainQueryResponse;
use prost::Message;

const CONTRACT_NAME: &str = concat!("crates.io:neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Reply ID used to tell this kind of reply call apart.
const REGISTER_BALANCES_ICQ_REPLY_ID: u64 = 1;

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
        ExecuteMsg::RegisterBalancesQuery {
            connection_id,
            addr,
            denoms,
            update_period,
        } => register_balances_query(env, connection_id, addr, denoms, update_period),
    }
}

pub fn register_balances_query(
    env: Env,
    connection_id: String,
    addr: String,
    denoms: Vec<String>,
    update_period: u64,
) -> NeutronResult<Response> {
    let msg = new_register_balances_query_msg(
        env.contract.address,
        connection_id,
        addr.clone(),
        denoms,
        update_period,
    )?;

    // Send the ICQ registration message as a submessage to receive a reply callback
    Ok(Response::new().add_submessage(SubMsg {
        id: REGISTER_BALANCES_ICQ_REPLY_ID,
        payload: to_json_binary(&addr)?,
        msg,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    match msg {
        QueryMsg::Balances { address } => query_balances(deps, address),
    }
}

pub fn query_balances(deps: Deps, addr: String) -> NeutronResult<Binary> {
    Ok(to_json_binary(&REMOTE_BALANCES.load(deps.storage, addr)?)?)
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> NeutronResult<Response> {
    match msg.id {
        REGISTER_BALANCES_ICQ_REPLY_ID => {
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
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> NeutronResult<Response> {
    match msg {
        SudoMsg::KVQueryResult { query_id } => sudo_kv_query_result(deps, env, query_id),
        _ => Ok(Response::default()),
    }
}

/// The contract's callback for KV query results. Note that only the query id is provided, so you
/// need to read the query result from the state.
pub fn sudo_kv_query_result(deps: DepsMut, env: Env, query_id: u64) -> NeutronResult<Response> {
    // Get the last submitted ICQ result from the Neutron ICQ module storage
    let balance_resp = query_balance(deps.as_ref(), env.clone(), query_id)?;
    // Get the address that was registered for the ICQ
    let addr = ICQ_ID_TO_WATCHED_ADDR.load(deps.storage, query_id)?;

    // Put your business logic here
    // For this example we just preserve the freshly fetched balances in the contract's state
    REMOTE_BALANCES.save(deps.storage, addr, &balance_resp.balances)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
