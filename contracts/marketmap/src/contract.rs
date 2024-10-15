use crate::query::QueryMsg;
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use neutron_std::types::slinky::marketmap::v1::MarketmapQuerier;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {}

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
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> StdResult<Response> {
    Ok(Default::default())
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    query_marketmap(deps, env, msg)
}

fn query_marketmap(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let querier = MarketmapQuerier::new(&deps.querier);
    match msg {
        QueryMsg::Params { .. } => to_json_binary(&querier.params()?),
        QueryMsg::LastUpdated { .. } => to_json_binary(&querier.last_updated()?),
        QueryMsg::MarketMap { .. } => to_json_binary(&querier.market_map()?),
        QueryMsg::Market { currency_pair } => to_json_binary(&querier.market(Some(currency_pair))?),
    }
}
