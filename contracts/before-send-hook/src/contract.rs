use crate::msg::{BlockBeforeSendMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg, SudoResResponse, TrackBeforeSendMsg};
use cosmwasm_std::{
    entry_point, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult,
};
use crate::state::{SUDO_RES_BLOCK, SUDO_RES_TRACK};

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
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<SudoResResponse> {
    match msg {
        QueryMsg::SudoResultBlockBefore {
        } => query_sudo_result_block_before(deps),
        QueryMsg::SudoResultTrackBefore {
        } => query_sudo_result_track_before(deps),
    }
}

#[entry_point]
pub fn sudo(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: SudoMsg,
) -> StdResult<Response> {
    match msg {
        SudoMsg::TrackBeforeSendSudoMsg {track_before_send_msg } => sudo_result_track_before(deps, track_before_send_msg),
        SudoMsg::BlockBeforeSendSudoMsg {block_before_send_msg} => sudo_result_block_before(deps, block_before_send_msg),
    }
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::new())
}

fn query_sudo_result_block_before(deps: Deps) -> StdResult<SudoResResponse> {
    let res = SUDO_RES_TRACK.load(deps.storage)?;
    let resp = SudoResResponse::Block { received: res };
    Ok(resp)
}

fn query_sudo_result_track_before(deps: Deps) -> StdResult<SudoResResponse> {
    let res = SUDO_RES_TRACK.load(deps.storage)?;
    let resp = SudoResResponse::Block { received: res };
    Ok(resp)
}

fn sudo_result_track_before(deps: DepsMut, _: TrackBeforeSendMsg) -> StdResult<Response> {
    SUDO_RES_TRACK.save(deps.storage, &true)?;
    Ok(Response::new())
}

fn sudo_result_block_before(deps: DepsMut, _: BlockBeforeSendMsg) -> StdResult<Response> {
    SUDO_RES_BLOCK.save(deps.storage, &true)?;
    Ok(Response::new())
}
