use cosmwasm_std::{
    entry_point, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult,
};
use cw2::set_contract_version;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}

use neutron_sdk::bindings::{
    dex::msg::DexMsg,
    dex::query::{
        AllInactiveLimitOrderTrancheResponse, AllLimitOrderTrancheResponse,
        AllLimitOrderTrancheUserResponse, AllPoolMetadataResponse, AllPoolReservesResponse,
        AllTickLiquidityResponse, AllUserDepositsResponse, AllUserLimitOrdersResponse, DexQuery,
        EstimateMultiHopSwapResponse, EstimatePlaceLimitOrderResponse,
        InactiveLimitOrderTrancheResponse, LimitOrderTrancheResponse,
        LimitOrderTrancheUserResponse, ParamsResponse, PoolMetadataResponse, PoolReservesResponse,
        PoolResponse, SimulateCancelLimitOrderResponse, SimulateDepositResponse,
        SimulateMultiHopSwapResponse, SimulatePlaceLimitOrderResponse,
        SimulateWithdrawFilledLimitOrderResponse, SimulateWithdrawalResponse,
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
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: DexMsg,
) -> StdResult<Response<NeutronMsg>> {
    deps.api
        .debug(format!("WASMDEBUG: execute: received msg: {:?}", msg).as_str());
    execute_dex(deps, env, info, msg)
}

fn execute_dex(
    _deps: DepsMut,
    _env: Env,
    _: MessageInfo,
    msg: DexMsg,
) -> StdResult<Response<NeutronMsg>> {
    Ok(Response::default().add_message(CosmosMsg::Custom(NeutronMsg::Dex(msg))))
}

#[entry_point]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: DexQuery) -> StdResult<Binary> {
    query_dex(deps, env, msg)
}

fn query_dex(deps: Deps<NeutronQuery>, _env: Env, msg: DexQuery) -> StdResult<Binary> {
    match msg {
        DexQuery::Params {} => {
            let query_response: ParamsResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::LimitOrderTrancheUser { .. } => {
            let query_response: LimitOrderTrancheUserResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::LimitOrderTrancheUserAll { .. } => {
            let query_response: AllLimitOrderTrancheUserResponse =
                deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::LimitOrderTrancheUserAllByAddress { .. } => {
            let query_response: AllUserLimitOrdersResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::LimitOrderTranche { .. } => {
            let query_response: LimitOrderTrancheResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::LimitOrderTrancheAll { .. } => {
            let query_response: AllLimitOrderTrancheResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::UserDepositAll { .. } => {
            let query_response: AllUserDepositsResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::TickLiquidityAll { .. } => {
            let query_response: AllTickLiquidityResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::InactiveLimitOrderTranche { .. } => {
            let query_response: InactiveLimitOrderTrancheResponse =
                deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::InactiveLimitOrderTrancheAll { .. } => {
            let query_response: AllInactiveLimitOrderTrancheResponse =
                deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::PoolReservesAll { .. } => {
            let query_response: AllPoolReservesResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::PoolReserves { .. } => {
            let query_response: PoolReservesResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::EstimateMultiHopSwap { .. } => {
            let query_response: EstimateMultiHopSwapResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::EstimatePlaceLimitOrder { .. } => {
            let query_response: EstimatePlaceLimitOrderResponse =
                deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::Pool { .. } => {
            let query_response: PoolResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::PoolByID { .. } => {
            let query_response: PoolResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::PoolMetadata { .. } => {
            let query_response: PoolMetadataResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::PoolMetadataAll { .. } => {
            let query_response: AllPoolMetadataResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::SimulateDeposit { .. } => {
            let query_response: SimulateDepositResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::SimulateWithdrawal { .. } => {
            let query_response: SimulateWithdrawalResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::SimulatePlaceLimitOrder { .. } => {
            let query_response: SimulatePlaceLimitOrderResponse =
                deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::SimulateWithdrawFilledLimitOrder { .. } => {
            let query_response: SimulateWithdrawFilledLimitOrderResponse =
                deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::SimulateCancelLimitOrder { .. } => {
            let query_response: SimulateCancelLimitOrderResponse =
                deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
        DexQuery::SimulateMultiHopSwap { .. } => {
            let query_response: SimulateMultiHopSwapResponse = deps.querier.query(&msg.into())?;
            to_json_binary(&query_response)
        }
    }
}
