use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use neutron_std::types::slinky::oracle::v1::OracleQuerier;
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
pub fn query(deps: Deps, env: Env, msg: OracleQuery) -> StdResult<Binary> {
    query_oracle(deps, env, msg)
}

fn query_oracle(deps: Deps, _env: Env, msg: OracleQuery) -> StdResult<Binary> {
    let querier = OracleQuerier::new(&deps.querier);
    match msg {
        OracleQuery::GetPrice { .. } => {
            // let query_response: GetPriceResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&querier.get_price(&query_response)?)
        }
        OracleQuery::GetPrices { .. } => {
            // let query_response: GetPricesResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&querier.get_prices())
        }
        OracleQuery::GetAllCurrencyPairs { .. } => {
            // let query_response: GetAllCurrencyPairsResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&querier.get_all_currency_pairs())
        }
    }
}
