// Copyright 2022 Neutron
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult,
};
use cw2::set_contract_version;

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use neutron_sdk::bindings::msg::{NeutronMsg, Unstake, UnstakeDescriptor};
use neutron_sdk::bindings::query::{IncentivesQuery, ModuleStatusResponse, NeutronQuery};
use neutron_sdk::NeutronResult;

const CONTRACT_NAME: &str = concat!("crates.io:neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    deps.api.debug("WASMDEBUG: instantiate");
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response<NeutronMsg>> {
    deps.api
        .debug(format!("WASMDEBUG: execute: received msg: {:?}", msg).as_str());
    match msg {
        ExecuteMsg::AddToGauge { gauge_id, rewards } => {
            execute_add_to_gauge(deps, env, gauge_id, rewards)
        }
        ExecuteMsg::Stake { coins } => execute_stake(deps, env, coins),
        ExecuteMsg::Unstake { unstakes } => execute_unstake(deps, env, unstakes),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    match msg {
        QueryMsg::ModuleStatus {} => query_module_status(deps, env),
        QueryMsg::GaugeByID { id } => query_gauge_by_id(deps, env, id),
        QueryMsg::Gauges { status, denom } => query_gauges(deps, env, status, denom),
        QueryMsg::StakeByID { stake_id } => query_stake_by_id(deps, env, stake_id),
        QueryMsg::Stakes { owner } => query_stakes(deps, env, owner),
    }
}

fn query_module_status(deps: Deps<NeutronQuery>, env: Env) -> NeutronResult<Binary> {
    let query = NeutronQuery::Incentives(IncentivesQuery::ModuleStatus {});

    let res: ModuleStatusResponse = deps.querier.query(&query.into())?;
    Ok(to_binary(&res)?)
}

fn query_gauge_by_id(deps: Deps<NeutronQuery>, env: Env, gauge_id: u64) -> NeutronResult<Binary> {
    todo!()
}

fn query_gauges(
    deps: Deps<NeutronQuery>,
    env: Env,
    status: String,
    denom: String,
) -> NeutronResult<Binary> {
    todo!()
}

fn query_stake_by_id(deps: Deps<NeutronQuery>, env: Env, stake_id: u64) -> NeutronResult<Binary> {
    todo!()
}

pub fn query_stakes(_deps: Deps<NeutronQuery>, _env: Env, owner: String) -> NeutronResult<Binary> {
    todo!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}

fn execute_add_to_gauge(
    _deps: DepsMut,
    _env: Env,
    gauge_id: u64,
    rewards: Vec<Coin>,
) -> StdResult<Response<NeutronMsg>> {
    todo!()
}
fn execute_stake(_deps: DepsMut, _env: Env, coins: Vec<Coin>) -> StdResult<Response<NeutronMsg>> {
    todo!()
}

fn execute_unstake(
    _deps: DepsMut,
    _env: Env,
    unstakes: Vec<UnstakeDescriptor>,
) -> StdResult<Response<NeutronMsg>> {
    todo!()
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    deps.api
        .debug(format!("WASMDEBUG: reply msg: {:?}", msg).as_str());
    match msg.id {
        _ => Err(StdError::generic_err(format!(
            "unsupported reply message id {}",
            msg.id
        ))),
    }
}
