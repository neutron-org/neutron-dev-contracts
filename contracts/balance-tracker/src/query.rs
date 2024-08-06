#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdResult, Uint128};

use astroport::tokenfactory_tracker::{ConfigResponse, QueryMsg};

use crate::state::{BALANCES, CONFIG, TOTAL_SUPPLY_HISTORY};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::BalanceAt { address, unit } => {
            to_json_binary(&balance_at(deps, env, address, unit)?)
        }
        QueryMsg::TotalSupplyAt { unit } => to_json_binary(&total_supply_at(deps, env, unit)?),
        QueryMsg::Config {} => {
            let config = CONFIG.load(deps.storage)?;
            to_json_binary(&ConfigResponse {
                tracked_denom: config.d,
                token_factory_module: config.m,
                track_over_seconds: true,
            })
        }
    }
}

fn balance_at(deps: Deps, env: Env, address: String, unit: Option<u64>) -> StdResult<Uint128> {
    let block_time = env.block.time.seconds();
    match unit.unwrap_or(block_time) {
        timestamp if timestamp == block_time => BALANCES.may_load(deps.storage, &address),
        timestamp => BALANCES.may_load_at_height(deps.storage, &address, timestamp),
    }
    .map(|balance| balance.unwrap_or_default())
}

fn total_supply_at(deps: Deps, env: Env, unit: Option<u64>) -> StdResult<Uint128> {
    let block_time = env.block.time.seconds();
    match unit.unwrap_or(block_time) {
        timestamp if timestamp == block_time => TOTAL_SUPPLY_HISTORY.may_load(deps.storage),
        timestamp => TOTAL_SUPPLY_HISTORY.may_load_at_height(deps.storage, timestamp),
    }
    .map(|total_supply| total_supply.unwrap_or_default())
}
