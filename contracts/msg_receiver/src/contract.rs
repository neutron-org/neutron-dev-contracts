use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, TestArg};
use cosmos_sdk_proto::cosmos::bank;
use cosmwasm_std::{
    entry_point, to_binary, Binary, ContractResult, Deps, DepsMut, Empty, Env, MessageInfo,
    QueryRequest, Response, StdError, StdResult, SystemResult,
};
use cw2::set_contract_version;
use neutron_sdk::bindings::msg::NeutronMsg;
use serde_json_wasm::to_vec;

use crate::state::TEST_ARGS;

const CONTRACT_NAME: &str = concat!("crates.io:neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response<NeutronMsg>> {
    deps.api
        .debug(format!("WASMDEBUG: execute: received msg: {:?}", msg).as_str());
    match msg {
        ExecuteMsg::TestMsg { return_err, arg } => execute_test_arg(deps, info, return_err, arg),
        ExecuteMsg::CallStaking {} => execute_call_staking(deps),
        ExecuteMsg::StargateMsg { address, denom } => {
            execute_stargate_msg(deps, info, address, denom)
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::TestMsg { arg } => to_binary(&query_test_msg(deps, env, arg)?),
    }
}

fn query_test_msg(deps: Deps, _env: Env, arg: String) -> StdResult<Option<TestArg>> {
    TEST_ARGS.may_load(deps.storage, &arg)
}

fn execute_test_arg(
    deps: DepsMut,
    info: MessageInfo,
    return_err: bool,
    arg: String,
) -> StdResult<Response<NeutronMsg>> {
    if return_err {
        return Err(StdError::generic_err("return error"));
    }

    TEST_ARGS.update::<_, StdError>(deps.storage, &arg, |maybe_t| match maybe_t {
        Some(t) => Ok(TestArg {
            sender: info.sender.to_string(),
            funds: info.funds,
            count: t.count + 1,
        }),
        None => Ok(TestArg {
            sender: info.sender.to_string(),
            funds: info.funds,
            count: 1,
        }),
    })?;

    Ok(Response::default().add_attribute("arg", arg))
}

fn execute_call_staking(deps: DepsMut) -> StdResult<Response<NeutronMsg>> {
    deps.querier.query_bonded_denom()?; // should fail since Neutron does not have staking module
    Ok(Response::default())
}

fn execute_stargate_msg(
    deps: DepsMut,
    _: MessageInfo,
    address: String,
    denom: String,
) -> StdResult<Response<NeutronMsg>> {
    let msg = bank::v1beta1::QueryBalanceRequest { address, denom };
    let resp: bank::v1beta1::QueryBalanceResponse =
        make_stargate_query::<bank::v1beta1::QueryBalanceResponse>(
            deps.as_ref(),
            "/cosmos.bank.v1beta1.Query/Balance".to_string(),
            ::prost::Message::encode_to_vec(&msg),
        )?;

    let balance = resp.balance.map(|b| b.amount).unwrap_or("0".to_string());

    Ok(Response::new().add_attribute("result", balance))
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}

pub fn make_stargate_query<T: ::prost::Message + Default>(
    deps: Deps,
    path: String,
    encoded_query_data: Vec<u8>,
) -> StdResult<T> {
    let raw = to_vec::<QueryRequest<Empty>>(&QueryRequest::Stargate {
        path,
        data: Binary::from(encoded_query_data),
    })
    .map_err(|serialize_err| {
        StdError::generic_err(format!("Serializing QueryRequest: {}", serialize_err))
    })?;
    match deps.querier.raw_query(&raw) {
        SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
            "Querier system error: {}",
            system_err
        ))),
        SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(format!(
            "Querier contract error: {}",
            contract_err
        ))),
        // response(value) is base64 encoded bytes
        SystemResult::Ok(ContractResult::Ok(value)) => T::decode(value.as_slice())
            .map_err(|e| StdError::generic_err(format!("Protobuf parsing error: {}", e))),
    }
}
