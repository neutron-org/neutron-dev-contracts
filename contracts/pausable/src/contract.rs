use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::PAUSED,
};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::NeutronResult;

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
    _: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::Pause {} => execute_update_pause(deps, true),
        ExecuteMsg::Unpause {} => execute_update_pause(deps, false),
    }
}

fn execute_update_pause(deps: DepsMut, paused: bool) -> StdResult<Response<NeutronMsg>> {
    PAUSED.save(deps.storage, &paused)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    deps.api
        .debug(format!("WASMDEBUG: execute: received msg: {:?}", msg).as_str());
    match msg {
        QueryMsg::State {} => query_paused(deps),
    }
}

fn query_paused(deps: Deps) -> NeutronResult<Binary> {
    let paused = PAUSED.load(deps.storage)?;
    Ok(to_json_binary(&paused)?)
}
