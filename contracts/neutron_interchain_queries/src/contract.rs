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

use cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    SubMsg,
};
use cw2::set_contract_version;
use prost::Message as ProstMessage;

use crate::integration_tests_mock_handlers::{set_query_mock, unset_query_mock};
use crate::msg::{
    ExecuteMsg, GetRecipientTxsResponse, GetTransfersAmountResponse, InstantiateMsg,
    KvCallbackStatsResponse, MigrateMsg, QueryMsg,
};
use crate::state::{
    IntegrationTestsQueryMock, QueryKind, Transfer, INTEGRATION_TESTS_QUERY_MOCK,
    KV_CALLBACK_STATS, KV_QUERY_ID_TO_CALLBACKS, RECIPIENT_TXS, TRANSFERS,
};
use cosmos_sdk_proto::cosmos::tx::v1beta1::{TxBody, TxRaw};
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::{NeutronQuery, QueryRegisteredQueryResponse};
use neutron_sdk::bindings::types::{Height, KVKey};
use neutron_sdk::interchain_queries::types::{
    QueryPayload, TransactionFilterItem, TransactionFilterOp, TransactionFilterValue,
};
use neutron_sdk::interchain_queries::v045::queries::{
    query_balance, query_bank_total, query_delegations, query_distribution_fee_pool,
    query_government_proposals, query_staking_validators,
};
use neutron_sdk::interchain_queries::v045::types::{
    Balances, COSMOS_SDK_TRANSFER_MSG_URL, RECIPIENT_FIELD,
};
use neutron_sdk::interchain_queries::v045::{
    new_register_balance_query_msg, new_register_bank_total_supply_query_msg,
    new_register_delegator_delegations_query_msg, new_register_distribution_fee_pool_query_msg,
    new_register_gov_proposal_query_msg, new_register_staking_validators_query_msg,
    new_register_transfers_query_msg,
};
use neutron_sdk::interchain_queries::{get_registered_query, query_kv_result};
use neutron_sdk::sudo::msg::SudoMsg;
use neutron_sdk::{NeutronError, NeutronResult};
use serde_json_wasm;

/// defines the incoming transfers limit to make a case of failed callback possible.
const MAX_ALLOWED_TRANSFER: u64 = 20000;
const MAX_ALLOWED_MESSAGES: usize = 20;

const CONTRACT_NAME: &str = concat!("crates.io:neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const BALANCES_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> NeutronResult<Response> {
    deps.api.debug("WASMDEBUG: instantiate");
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    _: MessageInfo,
    msg: ExecuteMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::RegisterBalanceQuery {
            connection_id,
            addr,
            denom,
            update_period,
        } => register_balance_query(connection_id, addr, denom, update_period),
        ExecuteMsg::RegisterBankTotalSupplyQuery {
            connection_id,
            denoms,
            update_period,
        } => register_bank_total_supply_query(connection_id, denoms, update_period),
        ExecuteMsg::RegisterDistributionFeePoolQuery {
            connection_id,
            update_period,
        } => register_distribution_fee_pool_query(connection_id, update_period),
        ExecuteMsg::RegisterGovernmentProposalsQuery {
            connection_id,
            proposals_ids,
            update_period,
        } => register_gov_proposal_query(connection_id, proposals_ids, update_period),
        ExecuteMsg::RegisterStakingValidatorsQuery {
            connection_id,
            validators,
            update_period,
        } => register_staking_validators_query(connection_id, validators, update_period),
        ExecuteMsg::RegisterDelegatorDelegationsQuery {
            connection_id,
            delegator,
            validators,
            update_period,
        } => register_delegations_query(connection_id, delegator, validators, update_period),
        ExecuteMsg::RegisterTransfersQuery {
            connection_id,
            recipient,
            update_period,
            min_height,
        } => register_transfers_query(connection_id, recipient, update_period, min_height),
        ExecuteMsg::UpdateInterchainQuery {
            query_id,
            new_keys,
            new_update_period,
            new_recipient,
        } => update_interchain_query(query_id, new_keys, new_update_period, new_recipient),
        ExecuteMsg::RemoveInterchainQuery { query_id } => remove_interchain_query(deps, query_id),
        ExecuteMsg::IntegrationTestsSetQueryMock {} => set_query_mock(deps),
        ExecuteMsg::IntegrationTestsUnsetQueryMock {} => unset_query_mock(deps),
        ExecuteMsg::IntegrationTestsRegisterQueryEmptyId { connection_id } => {
            register_query_empty_id(deps, env, connection_id)
        }
        ExecuteMsg::IntegrationTestsRegisterQueryEmptyPath { connection_id } => {
            register_query_empty_path(deps, env, connection_id)
        }
        ExecuteMsg::IntegrationTestsRegisterQueryEmptyKeys { connection_id } => {
            register_query_empty_keys(deps, env, connection_id)
        }
    }
}

pub fn register_balance_query(
    connection_id: String,
    addr: String,
    denom: String,
    update_period: u64,
) -> NeutronResult<Response<NeutronMsg>> {
    let msg = new_register_balance_query_msg(connection_id, addr, denom, update_period)?;
    // wrap into submessage to save {query_id, query_type} on reply that'll later be used to handle sudo kv callback
    let submsg = SubMsg::reply_on_success(msg, BALANCES_REPLY_ID);

    Ok(Response::default().add_submessage(submsg))
}

pub fn register_bank_total_supply_query(
    connection_id: String,
    denoms: Vec<String>,
    update_period: u64,
) -> NeutronResult<Response<NeutronMsg>> {
    let msg = new_register_bank_total_supply_query_msg(connection_id, denoms, update_period)?;

    Ok(Response::new().add_message(msg))
}

pub fn register_distribution_fee_pool_query(
    connection_id: String,
    update_period: u64,
) -> NeutronResult<Response<NeutronMsg>> {
    let msg = new_register_distribution_fee_pool_query_msg(connection_id, update_period)?;

    Ok(Response::new().add_message(msg))
}

pub fn register_gov_proposal_query(
    connection_id: String,
    proposals_ids: Vec<u64>,
    update_period: u64,
) -> NeutronResult<Response<NeutronMsg>> {
    let msg = new_register_gov_proposal_query_msg(connection_id, proposals_ids, update_period)?;

    Ok(Response::new().add_message(msg))
}

pub fn register_staking_validators_query(
    connection_id: String,
    validators: Vec<String>,
    update_period: u64,
) -> NeutronResult<Response<NeutronMsg>> {
    let msg = new_register_staking_validators_query_msg(connection_id, validators, update_period)?;

    Ok(Response::new().add_message(msg))
}

pub fn register_delegations_query(
    connection_id: String,
    delegator: String,
    validators: Vec<String>,
    update_period: u64,
) -> NeutronResult<Response<NeutronMsg>> {
    let msg = new_register_delegator_delegations_query_msg(
        connection_id,
        delegator,
        validators,
        update_period,
    )?;

    Ok(Response::new().add_message(msg))
}

pub fn register_transfers_query(
    connection_id: String,
    recipient: String,
    update_period: u64,
    min_height: Option<u64>,
) -> NeutronResult<Response<NeutronMsg>> {
    let msg =
        new_register_transfers_query_msg(connection_id, recipient, update_period, min_height)?;

    Ok(Response::new().add_message(msg))
}

pub fn register_query_empty_id(
    _: DepsMut<NeutronQuery>,
    _: Env,
    connection_id: String,
) -> NeutronResult<Response<NeutronMsg>> {
    let kv_key = KVKey {
        path: "test".to_string(),
        key: Binary(vec![]),
    };
    let msg =
        NeutronMsg::register_interchain_query(QueryPayload::KV(vec![kv_key]), connection_id, 10)?;

    Ok(Response::new().add_message(msg))
}

pub fn register_query_empty_path(
    _: DepsMut<NeutronQuery>,
    _: Env,
    connection_id: String,
) -> NeutronResult<Response<NeutronMsg>> {
    let kv_key = KVKey {
        path: "".to_string(),
        key: Binary("test".as_bytes().to_vec()),
    };
    let msg =
        NeutronMsg::register_interchain_query(QueryPayload::KV(vec![kv_key]), connection_id, 10)?;
    Ok(Response::new().add_message(msg))
}

pub fn register_query_empty_keys(
    _: DepsMut<NeutronQuery>,
    _: Env,
    connection_id: String,
) -> NeutronResult<Response<NeutronMsg>> {
    let msg = NeutronMsg::register_interchain_query(QueryPayload::KV(vec![]), connection_id, 10)?;
    Ok(Response::new().add_message(msg))
}

pub fn update_interchain_query(
    query_id: u64,
    new_keys: Option<Vec<KVKey>>,
    new_update_period: Option<u64>,
    new_recipient: Option<String>,
) -> NeutronResult<Response<NeutronMsg>> {
    let new_filter = new_recipient.map(|recipient| {
        vec![TransactionFilterItem {
            field: RECIPIENT_FIELD.to_string(),
            op: TransactionFilterOp::Eq,
            value: TransactionFilterValue::String(recipient),
        }]
    });

    let update_msg =
        NeutronMsg::update_interchain_query(query_id, new_keys, new_update_period, new_filter)?;
    Ok(Response::new().add_message(update_msg))
}

pub fn remove_interchain_query(
    deps: DepsMut<NeutronQuery>,
    query_id: u64,
) -> NeutronResult<Response<NeutronMsg>> {
    let remove_msg = NeutronMsg::remove_interchain_query(query_id);
    KV_QUERY_ID_TO_CALLBACKS.remove(deps.storage, query_id);
    Ok(Response::new().add_message(remove_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    match msg {
        //TODO: check if query.result.height is too old (for all interchain queries)
        QueryMsg::Balance { query_id } => Ok(to_binary(&query_balance(deps, env, query_id)?)?),
        QueryMsg::BankTotalSupply { query_id } => {
            Ok(to_binary(&query_bank_total(deps, env, query_id)?)?)
        }
        QueryMsg::DistributionFeePool { query_id } => Ok(to_binary(&query_distribution_fee_pool(
            deps, env, query_id,
        )?)?),
        QueryMsg::StakingValidators { query_id } => {
            Ok(to_binary(&query_staking_validators(deps, env, query_id)?)?)
        }
        QueryMsg::GovernmentProposals { query_id } => Ok(to_binary(&query_government_proposals(
            deps, env, query_id,
        )?)?),
        QueryMsg::GetDelegations { query_id } => {
            Ok(to_binary(&query_delegations(deps, env, query_id)?)?)
        }
        QueryMsg::GetRegisteredQuery { query_id } => {
            Ok(to_binary(&get_registered_query(deps, query_id)?)?)
        }
        QueryMsg::GetRecipientTxs { recipient } => query_recipient_txs(deps, recipient),
        QueryMsg::GetTransfersNumber {} => query_transfers_number(deps),
        QueryMsg::KvCallbackStats { query_id } => query_kv_callback_stats(deps, query_id),
    }
}

fn query_recipient_txs(deps: Deps<NeutronQuery>, recipient: String) -> NeutronResult<Binary> {
    let txs = RECIPIENT_TXS
        .load(deps.storage, &recipient)
        .unwrap_or_default();
    Ok(to_binary(&GetRecipientTxsResponse { transfers: txs })?)
}

/// Returns the number of transfers made on remote chain and queried with ICQ
fn query_transfers_number(deps: Deps<NeutronQuery>) -> NeutronResult<Binary> {
    let transfers_number = TRANSFERS.load(deps.storage).unwrap_or_default();
    Ok(to_binary(&GetTransfersAmountResponse { transfers_number })?)
}

/// Returns block height of last KV query callback execution
pub fn query_kv_callback_stats(deps: Deps<NeutronQuery>, query_id: u64) -> NeutronResult<Binary> {
    Ok(to_binary(&KvCallbackStatsResponse {
        last_update_height: KV_CALLBACK_STATS
            .may_load(deps.storage, query_id)?
            .unwrap_or(0),
    })?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}

#[entry_point]
pub fn sudo(deps: DepsMut<NeutronQuery>, env: Env, msg: SudoMsg) -> NeutronResult<Response> {
    match msg {
        SudoMsg::TxQueryResult {
            query_id,
            height,
            data,
        } => sudo_tx_query_result(deps, env, query_id, height, data),
        SudoMsg::KVQueryResult { query_id } => sudo_kv_query_result(deps, env, query_id),
        _ => Ok(Response::default()),
    }
}

/// sudo_check_tx_query_result is an example callback for transaction query results that stores the
/// deposits received as a result on the registered query in the contract's state.
pub fn sudo_tx_query_result(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    query_id: u64,
    _height: Height,
    data: Binary,
) -> NeutronResult<Response> {
    if let Some(IntegrationTestsQueryMock::Enabled {}) =
        INTEGRATION_TESTS_QUERY_MOCK.may_load(deps.storage)?
    {
        // simulate error on tx query submit for integration tests
        return Err(NeutronError::IntegrationTestsMock {});
    }
    // Decode the transaction data
    let tx: TxRaw = TxRaw::decode(data.as_slice())?;
    let body: TxBody = TxBody::decode(tx.body_bytes.as_slice())?;

    // Get the registered query by ID and retrieve the raw query string
    let registered_query: QueryRegisteredQueryResponse =
        get_registered_query(deps.as_ref(), query_id)?;
    let transactions_filter = registered_query.registered_query.transactions_filter;

    #[allow(clippy::match_single_binding)]
    // Depending of the query type, check the transaction data to see whether is satisfies
    // the original query. If you don't write specific checks for a transaction query type,
    // all submitted results will be treated as valid.
    //
    // TODO: come up with solution to determine transactions filter type
    match registered_query.registered_query.query_type {
        _ => {
            // For transfer queries, query data looks like `[{"field:"transfer.recipient", "op":"eq", "value":"some_address"}]`
            let query_data: Vec<TransactionFilterItem> =
                serde_json_wasm::from_str(transactions_filter.as_str()).map_err(|e| {
                    StdError::generic_err(format!("failed to parse transactions_filter: {:?}", e))
                })?;

            let recipient = query_data
                .iter()
                .find(|x| x.field == RECIPIENT_FIELD && x.op == TransactionFilterOp::Eq)
                .map(|x| match &x.value {
                    TransactionFilterValue::String(v) => v.as_str(),
                    _ => "",
                })
                .unwrap_or("");

            let deposits = recipient_deposits_from_tx_body(body, recipient)?;
            // If we didn't find a Send message with the correct recipient, return an error, and
            // this query result will be rejected by Neutron: no data will be saved to state.
            if deposits.is_empty() {
                return Err(NeutronError::Std(StdError::generic_err(
                    "failed to find a matching transaction message",
                )));
            }

            let mut stored_transfers: u64 = TRANSFERS.load(deps.storage).unwrap_or_default();
            stored_transfers += deposits.len() as u64;
            TRANSFERS.save(deps.storage, &stored_transfers)?;

            check_deposits_size(&deposits)?;
            let mut stored_deposits: Vec<Transfer> = RECIPIENT_TXS
                .load(deps.storage, recipient)
                .unwrap_or_default();
            stored_deposits.extend(deposits);
            RECIPIENT_TXS.save(deps.storage, recipient, &stored_deposits)?;
            Ok(Response::new())
        }
    }
}

/// parses tx body and retrieves transactions to the given recipient.
fn recipient_deposits_from_tx_body(
    tx_body: TxBody,
    recipient: &str,
) -> NeutronResult<Vec<Transfer>> {
    let mut deposits: Vec<Transfer> = vec![];
    // Only handle up to MAX_ALLOWED_MESSAGES messages, everything else
    // will be ignored to prevent 'out of gas' conditions.
    // Note: in real contracts you will have to somehow save ignored
    // data in order to handle it later.
    for msg in tx_body.messages.iter().take(MAX_ALLOWED_MESSAGES) {
        // Skip all messages in this transaction that are not Send messages.
        if msg.type_url != *COSMOS_SDK_TRANSFER_MSG_URL.to_string() {
            continue;
        }

        // Parse a Send message and check that it has the required recipient.
        let transfer_msg: MsgSend = MsgSend::decode(msg.value.as_slice())?;
        if transfer_msg.to_address == recipient {
            for coin in transfer_msg.amount {
                deposits.push(Transfer {
                    sender: transfer_msg.from_address.clone(),
                    amount: coin.amount.clone(),
                    denom: coin.denom,
                    recipient: recipient.to_string(),
                });
            }
        }
    }
    Ok(deposits)
}

// checks whether there are deposits that are greater then MAX_ALLOWED_TRANSFER.
fn check_deposits_size(deposits: &Vec<Transfer>) -> StdResult<()> {
    for deposit in deposits {
        match deposit.amount.parse::<u64>() {
            Ok(amount) => {
                if amount > MAX_ALLOWED_TRANSFER {
                    return Err(StdError::generic_err(format!(
                        "maximum allowed transfer is {}",
                        MAX_ALLOWED_TRANSFER
                    )));
                };
            }
            Err(error) => {
                return Err(StdError::generic_err(format!(
                    "failed to cast transfer amount to u64: {}",
                    error
                )));
            }
        };
    }
    Ok(())
}

/// sudo_kv_query_result is the contract's callback for KV query results. Note that only the query
/// id is provided, so you need to read the query result from the state.
pub fn sudo_kv_query_result(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    query_id: u64,
) -> NeutronResult<Response> {
    deps.api.debug(
        format!(
            "WASMDEBUG: sudo_kv_query_result received; query_id: {:?}",
            query_id,
        )
        .as_str(),
    );

    if let Some(IntegrationTestsQueryMock::Enabled {}) =
        INTEGRATION_TESTS_QUERY_MOCK.may_load(deps.storage)?
    {
        // doesn't really matter whatever data we try to save here, it should all be reverted
        // since we return an error in this branch anyway. in fact, this branch exists for the
        // sole reason of testing this particular revert behaviour.
        KV_CALLBACK_STATS.save(deps.storage, query_id, &0)?;
        return Err(NeutronError::IntegrationTestsMock {});
    }

    // store last KV callback update time
    KV_CALLBACK_STATS.save(deps.storage, query_id, &env.block.height)?;

    let query_kind = KV_QUERY_ID_TO_CALLBACKS.may_load(deps.storage, query_id)?;
    match query_kind {
        Some(QueryKind::Balance) => {
            let balances: Balances = query_kv_result(deps.as_ref(), query_id)?;
            let balances_str = balances
                .coins
                .iter()
                .map(|c| c.amount.to_string() + c.denom.as_str())
                .collect::<Vec<String>>()
                .join(", ");
            deps.api
                .debug(format!("WASMDEBUG: sudo callback; balances: {:?}", balances_str).as_str());
        }
        None => {
            deps.api.debug(
                format!(
                    "WASMDEBUG: sudo callback without query kind assigned; query_id: {:?}",
                    query_id
                )
                .as_str(),
            );
        }
    }
    Ok(Response::default())
}
