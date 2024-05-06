use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult,
};
use cw2::set_contract_version;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}

use neutron_sdk::bindings::{
    oracle::query::{ OracleQuery, GetPricesResponse, GetPriceResponse, GetAllCurrencyPairsResponse
    },
    msg::NeutronMsg,
    query::NeutronQuery,
};

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
) -> StdResult<Response<NeutronMsg>> {
    Ok(Default::default())
}

#[entry_point]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: OracleQuery) -> StdResult<Binary> {
    query_oracle(deps, env, msg)
}

fn query_oracle(deps: Deps<NeutronQuery>, _env: Env, msg: OracleQuery) -> StdResult<Binary> {
    match msg {
        OracleQuery::GetPriceRequest { .. } => {
            let query_response: GetPriceResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        OracleQuery::GetPricesRequest { .. } => {
            let query_response: GetPricesResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        OracleQuery::GetAllCurrencyPairs { .. } => {
            let query_response: GetAllCurrencyPairsResponse =
                deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
    }
}
