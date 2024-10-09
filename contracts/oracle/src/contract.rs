use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {}

use neutron_sdk::bindings::{msg::NeutronMsg, oracle::query::OracleQuery, query::NeutronQuery};

use neutron_std::types::connect::oracle::v2::OracleQuerier;

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
) -> StdResult<Response<NeutronMsg>> {
    Ok(Default::default())
}

#[entry_point]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: OracleQuery) -> StdResult<Binary> {
    query_oracle(deps, env, msg)
}

fn query_oracle(deps: Deps<NeutronQuery>, _env: Env, msg: OracleQuery) -> StdResult<Binary> {
    let oracle_querier = OracleQuerier::new(&deps.querier);
    match msg {
        OracleQuery::GetPrice { currency_pair } => to_json_binary(
            &oracle_querier.get_price(format!("{}/{}", currency_pair.base, currency_pair.quote))?,
        ),
        OracleQuery::GetPrices { currency_pair_ids } => {
            to_json_binary(&oracle_querier.get_prices(currency_pair_ids)?)
        }
        OracleQuery::GetAllCurrencyPairs { .. } => {
            to_json_binary(&oracle_querier.get_all_currency_pairs()?)
        }
    }
}
