use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{PriceFeedRate, ERROR, RATES};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;

// Version info for migration
const CONTRACT_NAME: &str = "crates.io:band-ibc-price-feed";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Request {} => try_request(deps, env),
        ExecuteMsg::SetRate { symbol, rate } => set_rate(deps, env, symbol, rate),
    }
}

pub fn try_request(_deps: DepsMut, _env: Env) -> Result<Response, ContractError> {
    Ok(Response::default())
}

pub fn set_rate(
    deps: DepsMut,
    _env: Env,
    symbol: String,
    rate: PriceFeedRate,
) -> Result<Response, ContractError> {
    RATES.save(deps.storage, &symbol, &rate)?;
    Ok(Response::default())
}

/// this is a no-op
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> StdResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetRate {} => to_binary(&query_rate(deps)?),
        QueryMsg::GetError {} => to_binary(&query_error(deps)?),
    }
}

fn query_error(deps: Deps) -> StdResult<String> {
    ERROR.load(deps.storage)
}

fn query_rate(deps: Deps) -> StdResult<Vec<PriceFeedRate>> {
    let out = RATES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|x| x.unwrap().1)
        .collect::<Vec<PriceFeedRate>>();
    Ok(out)
}
