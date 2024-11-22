use crate::query::{ChainResponse, InterchainQueries, QueryMsg};
use cosmwasm_std::{entry_point, to_json_binary, to_json_vec, BankMsg, Binary, ContractResult, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest, Reply, Response, StdError, StdResult, SubMsg, SystemResult, Uint128};
use cw2::set_contract_version;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}
use neutron_sdk::sudo::msg::SudoMsg;

const REFLECT_REPLY_ID: u64 = 0;

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Send { to: String, amount: Uint128 },
    ReflectMsg { msgs: Vec<CosmosMsg> },
    Burn {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    deps.api
        .debug(format!("WASMDEBUG: execute: received msg: {:?}", msg).as_str());
    match msg {
        ExecuteMsg::Send { .. } => {
            unimplemented!()
        }
        ExecuteMsg::ReflectMsg { msgs } => {
            let submsgs = msgs
                .into_iter()
                .map(|m| SubMsg::reply_on_success(m, REFLECT_REPLY_ID));

            Ok(Response::default().add_submessages(submsgs))
        }
        ExecuteMsg::Burn {  } => burn_tokens(deps, env, info)
    }
}

pub fn burn_tokens(_deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, StdError> {
    let funds = info.funds;

    let msg = BankMsg::Burn { amount: funds };

    Ok(Response::new().add_message(msg))

}

#[entry_point]
pub fn query(deps: Deps<InterchainQueries>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Reflect(payload) => to_json_binary(&query_with_payload(deps, env, payload)?),
    }
}

#[entry_point]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        REFLECT_REPLY_ID => {
            let res = Response::default();

            let msg_responses = msg.result.unwrap().msg_responses;
            if msg_responses.is_empty() {
                Ok(res)
            } else {
                Ok(res.set_data(msg_responses[0].clone().value))
            }
        }
        _ => {
            unimplemented!()
        }
    }
}

fn query_with_payload(
    deps: Deps<InterchainQueries>,
    _env: Env,
    icq_query: QueryRequest<InterchainQueries>,
) -> StdResult<ChainResponse> {
    let raw = to_json_vec(&icq_query).map_err(|serialize_err| {
        StdError::generic_err(format!("Serializing QueryRequest: {}", serialize_err))
    })?;
    match deps.querier.raw_query(&raw) {
        SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
            "Querier system error: {}",
            system_err
        ))),
        SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(format!(
            "Querier contract error: {}",
            contract_err
        ))),
        SystemResult::Ok(ContractResult::Ok(value)) => Ok(ChainResponse { data: value }),
    }
}

#[entry_point]
pub fn sudo(_deps: DepsMut, _env: Env, _msg: SudoMsg) -> StdResult<Response> {
    Ok(Response::default())
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
