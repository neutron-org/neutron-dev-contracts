use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{BEGIN_BLOCKER_SCHEDULES, END_BLOCKER_SCHEDULES};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult,
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = concat!("crates.io:neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MODULE_ACCOUNT: &str = "neutron1cd6wafvehv79pm2yxth40thpyc7dc0yrqkyk95";

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
pub fn execute(deps: DepsMut, _: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    deps.api
        .debug(format!("WASMDEBUG: execute: received msg: {:?}", msg).as_str());

    if info.sender.as_str() != MODULE_ACCOUNT {
        return Err(StdError::generic_err("Unauthorized"));
    }

    match msg {
        ExecuteMsg::AddBeginBlockerSchedule { name } => {
            let counter = BEGIN_BLOCKER_SCHEDULES
                .may_load(deps.storage, name.clone())?
                .unwrap_or_default()
                .checked_add(1)
                .unwrap_or_default();

            BEGIN_BLOCKER_SCHEDULES.save(deps.storage, name, &counter)?;

            Ok(Response::default())
        }
        ExecuteMsg::AddEndBlockerSchedule { name } => {
            let counter = END_BLOCKER_SCHEDULES
                .may_load(deps.storage, name.clone())?
                .unwrap_or_default()
                .checked_add(1)
                .unwrap_or_default();

            END_BLOCKER_SCHEDULES.save(deps.storage, name, &counter)?;

            Ok(Response::default())
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetBeginBlockerScheduleCounter { name } => {
            let res = BEGIN_BLOCKER_SCHEDULES.may_load(deps.storage, name)?;
            to_json_binary(&res)
        }
        QueryMsg::GetEndBlockerScheduleCounter { name } => {
            let res = END_BLOCKER_SCHEDULES.may_load(deps.storage, name)?;
            to_json_binary(&res)
        }
    }
}
