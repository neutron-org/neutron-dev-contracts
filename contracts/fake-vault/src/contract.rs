use crate::{
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    store::{Config, CONFIG, USERS},
};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = concat!("crates.io:neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let config = Config {
        dao: msg.dao,
        description: msg.description,
        info: msg.info,
        name: msg.name,
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(deps: DepsMut, _env: Env, _: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::SetPower { address, power } => set_power(deps, address, power),
    }
}

fn set_power(deps: DepsMut, address: String, power: u64) -> StdResult<Response> {
    USERS.save(deps.storage, address, &power)?;
    Ok(Response::default())
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Dao {} => to_binary(&query_dao(deps)?),
        QueryMsg::Description {} => to_binary(&query_description(deps)?),
        QueryMsg::Info {} => to_binary(&query_info(deps)?),
        QueryMsg::Name {} => to_binary(&query_name(deps)?),
        QueryMsg::VotingPowerAtHeight { height, address } => {
            to_binary(&query_voting_power_at_height(deps, env, height, address)?)
        }
        QueryMsg::TotalPowerAtHeight { height } => {
            to_binary(&query_total_power_at_height(deps, env, height)?)
        }
    }
}

fn query_dao(deps: Deps) -> StdResult<String> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config.dao)
}

fn query_info(deps: Deps) -> StdResult<String> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config.info)
}

fn query_description(deps: Deps) -> StdResult<String> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config.description)
}
fn query_name(deps: Deps) -> StdResult<String> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config.name)
}

fn query_voting_power_at_height(
    deps: Deps,
    _env: Env,
    _height: Option<u64>,
    address: String,
) -> StdResult<u64> {
    let power = USERS.may_load(deps.storage, address)?.unwrap_or_default();
    Ok(power)
}

fn query_total_power_at_height(deps: Deps, _env: Env, _height: Option<u64>) -> StdResult<u64> {
    let mut total_power: u64 = 0;
    for user in USERS.range(deps.storage, None, None, Order::Ascending) {
        let (_, power) = user?;
        total_power += power;
    }
    Ok(total_power)
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
