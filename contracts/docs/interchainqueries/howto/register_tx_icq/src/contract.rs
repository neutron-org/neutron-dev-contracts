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

use std::str::FromStr;

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::UNDELEGATED_AMOUNTS;
use cosmos_sdk_proto::cosmos::staking::v1beta1::MsgUndelegate;
use cosmos_sdk_proto::cosmos::tx::v1beta1::{TxBody, TxRaw};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdError, StdResult, Uint128,
};
use cw2::set_contract_version;
use neutron_sdk::interchain_queries::{
    get_registered_query,
    types::{QueryType, TransactionFilterItem, TransactionFilterOp, TransactionFilterValue},
};
use neutron_sdk::sudo::msg::{Height, SudoMsg};
use neutron_sdk::{NeutronError, NeutronResult};
use neutron_std::types::neutron::interchainqueries::{MsgRegisterInterchainQuery, RegisteredQuery};
use prost::Message;
use serde_json_wasm::to_string;

const CONTRACT_NAME: &str = concat!("crates.io:neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Unbond delegator attribute key.
/// https://github.com/cosmos/cosmos-sdk/blob/8bfcf554275c1efbb42666cc8510d2da139b67fa/x/staking/keeper/msg_server.go#L447-L455
const UNBOND_DELEGATOR_ATTR: &str = "unbond.delegator";
const STAKING_UNDELEGATE_MSG_URL: &str = "/cosmos.staking.v1beta1.MsgUndelegate";

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
        ExecuteMsg::RegisterUndelegationsQuery {
            connection_id,
            addr,
            update_period,
        } => register_undelegations_query(env, connection_id, addr, update_period),
    }
}

pub fn register_undelegations_query(
    env: Env,
    connection_id: String,
    addr: String,
    update_period: u64,
) -> NeutronResult<Response> {
    let msg = MsgRegisterInterchainQuery {
        query_type: QueryType::TX.into(),
        keys: vec![],
        // the result filter is unbond.delegator=addr
        transactions_filter: to_string(&vec![TransactionFilterItem {
            field: UNBOND_DELEGATOR_ATTR.to_string(),
            op: TransactionFilterOp::Eq,
            value: TransactionFilterValue::String(addr.clone()),
        }])
        .map_err(|e| StdError::generic_err(e.to_string()))?,
        connection_id,
        update_period,
        sender: env.contract.address.to_string(),
    };

    Ok(Response::default().add_message(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    match msg {
        QueryMsg::UndelegatedAmount { address } => query_undelegated_amount(deps, address),
    }
}

pub fn query_undelegated_amount(deps: Deps, addr: String) -> NeutronResult<Binary> {
    Ok(to_json_binary(
        &UNDELEGATED_AMOUNTS
            .may_load(deps.storage, addr)?
            .unwrap_or_default(),
    )?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

#[entry_point]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> NeutronResult<Response> {
    match msg {
        SudoMsg::TxQueryResult {
            query_id,
            height,
            data,
        } => sudo_tx_query_result(deps, env, query_id, height, data),
        _ => Ok(Response::default()),
    }
}

#[entry_point]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> NeutronResult<Response> {
    Ok(Response::default())
}

/// The contract's callback for TX query results.
pub fn sudo_tx_query_result(
    deps: DepsMut,
    _env: Env,
    query_id: u64,
    _height: Height,
    data: Binary,
) -> NeutronResult<Response> {
    // Decode the transaction data
    let tx: TxRaw = TxRaw::decode(data.as_slice())?;
    let body: TxBody = TxBody::decode(tx.body_bytes.as_slice())?;

    // Get the registered query by ID and retrieve the delegator address from query's transaction filter
    let registered_query: RegisteredQuery = get_registered_query(deps.as_ref(), query_id)?;
    let query_tx_filter: Vec<TransactionFilterItem> =
        serde_json_wasm::from_str(registered_query.transactions_filter.as_str())?;
    let delegator = match &query_tx_filter[0].value {
        TransactionFilterValue::String(s) => s.clone(),
        _ => {
            return Err(NeutronError::Std(StdError::generic_err(
                "undelegations transaction filter value must be a String",
            )))
        }
    };

    let mut new_undelegations: Vec<Coin> = vec![];
    for msg in body.messages.iter() {
        // Narrow down the messages to only MsgUndelegate ones
        if msg.type_url != STAKING_UNDELEGATE_MSG_URL {
            continue;
        }
        // Narrow down the MsgUndelegate messages to only those that match the delegator address
        let undelegate_msg = MsgUndelegate::decode(msg.value.as_slice())?;
        if undelegate_msg.delegator_address != delegator {
            continue;
        }

        #[allow(clippy::unwrap_used)]
        let undelegation_amount = undelegate_msg.amount.unwrap();
        new_undelegations.push(Coin {
            denom: undelegation_amount.denom,
            amount: Uint128::from_str(undelegation_amount.amount.as_str())?,
        });
    }

    if !new_undelegations.is_empty() {
        let mut undelegations = UNDELEGATED_AMOUNTS
            .may_load(deps.storage, delegator.clone())?
            .unwrap_or_default();
        undelegations.extend(new_undelegations);
        UNDELEGATED_AMOUNTS.save(deps.storage, delegator, &undelegations)?;
    }

    Ok(Response::default())
}
