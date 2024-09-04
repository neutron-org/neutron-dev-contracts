use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{BEGIN_BLOCKER_SHEDULES, END_BLOCKER_SHEDULES};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

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
pub fn execute(deps: DepsMut, _: Env, _: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    deps.api
        .debug(format!("WASMDEBUG: execute: received msg: {:?}", msg).as_str());

    match msg {
        ExecuteMsg::BeginBlocker { name } => {
            let counter = BEGIN_BLOCKER_SHEDULES
                .load(deps.storage, name.clone())?
                .checked_add(1)
                .unwrap_or_default();

            BEGIN_BLOCKER_SHEDULES.save(deps.storage, name, &counter)?;

            Ok(Response::default())
        }
        ExecuteMsg::EndBlocker { name } => {
            let counter = END_BLOCKER_SHEDULES
                .load(deps.storage, name.clone())?
                .checked_add(1)
                .unwrap_or_default();

            END_BLOCKER_SHEDULES.save(deps.storage, name, &counter)?;

            Ok(Response::default())
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetBeginBlockerScheduleCounter { name } => {
            let res = BEGIN_BLOCKER_SHEDULES.load(deps.storage, name)?;
            to_json_binary(&res)
        }
        QueryMsg::GetEndBlockerScheduleCounter { name } => {
            let res = END_BLOCKER_SHEDULES.load(deps.storage, name)?;
            to_json_binary(&res)
        }
    }
}
