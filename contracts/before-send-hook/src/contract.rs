use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg, SudoResResponse};
use crate::state::{SUDO_RES_BLOCK, SUDO_RES_TRACK};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    SUDO_RES_TRACK.save(deps.storage, &false)?;
    SUDO_RES_BLOCK.save(deps.storage, &false)?;

    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> StdResult<Response> {
    Ok(Response::new())
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::SudoResultBlockBefore {} => {
            to_json_binary(&query_sudo_result_block_before(deps)?)
        }
        QueryMsg::SudoResultTrackBefore {} => {
            to_json_binary(&query_sudo_result_track_before(deps)?)
        }
    }
}

#[entry_point]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> StdResult<Response> {
    match msg {
        SudoMsg::TrackBeforeSend { .. } => sudo_result_track_before(deps),
        SudoMsg::BlockBeforeSend { .. } => sudo_result_block_before(deps),
    }
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::new())
}

fn query_sudo_result_block_before(deps: Deps) -> StdResult<SudoResResponse> {
    let res = SUDO_RES_BLOCK.load(deps.storage)?;
    let resp = SudoResResponse::Block { received: res };
    Ok(resp)
}

fn query_sudo_result_track_before(deps: Deps) -> StdResult<SudoResResponse> {
    let res = SUDO_RES_TRACK.load(deps.storage)?;
    let resp = SudoResResponse::Track { received: res };
    Ok(resp)
}

fn sudo_result_track_before(deps: DepsMut) -> StdResult<Response> {
    SUDO_RES_TRACK.save(deps.storage, &true)?;
    Ok(Response::new())
}

fn sudo_result_block_before(deps: DepsMut) -> StdResult<Response> {
    SUDO_RES_BLOCK.save(deps.storage, &true)?;
    Ok(Response::new())
}
