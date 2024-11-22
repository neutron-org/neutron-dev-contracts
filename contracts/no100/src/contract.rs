use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg};
use cosmwasm_std::{entry_point, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128};

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
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
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    Ok(Default::default())
}

#[entry_point]
pub fn sudo(_deps: DepsMut, _env: Env, msg: SudoMsg) -> StdResult<Response> {
    match msg {
        SudoMsg::TrackBeforeSend { amount, .. } => sudo_result_track_before(amount),
        SudoMsg::BlockBeforeSend { amount, .. } => sudo_result_block_before(amount),
    }
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::new())
}

fn sudo_result_track_before(amount: Coin) -> StdResult<Response> {
    if amount.amount >= Uint128::from(100u8) {
        return Err(StdError::generic_err("sending >100 tokens is not allowed"))
    }

    Ok(Response::new())
}

fn sudo_result_block_before(amount: Coin) -> StdResult<Response> {
    if amount.amount >= Uint128::from(100u8) {
        return Err(StdError::generic_err("sending >100 tokens is not allowed"))
    }

    Ok(Response::new())
}
