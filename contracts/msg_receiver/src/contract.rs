use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, TestArg};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};

use cw2::set_contract_version;
use neutron_sdk::bindings::msg::NeutronMsg;

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

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
