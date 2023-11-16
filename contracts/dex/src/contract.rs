use std::str::from_utf8;
use crate::query::{DexQuery};
use cosmwasm_std::{entry_point, to_binary, to_vec, Binary, ContractResult, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response, StdError, StdResult, SystemResult, Empty};
use cw2::set_contract_version;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use prost::Message;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}
use neutron_sdk::{
    bindings::{
        msg::{NeutronMsg},
        query::{NeutronQuery,
        },
    },
    sudo::msg::SudoMsg,
};

use neutron_sdk::proto_types::neutron::dex;
use neutron_sdk::proto_types::neutron::dex::{QueryAllInactiveLimitOrderTrancheRequest, QueryAllLimitOrderTrancheRequest, QueryAllLimitOrderTrancheUserRequest, QueryAllPoolMetadataRequest, QueryAllPoolReservesRequest, QueryAllTickLiquidityRequest, QueryAllUserDepositsRequest, QueryAllUserLimitOrdersRequest, QueryEstimateMultiHopSwapRequest, QueryEstimatePlaceLimitOrderRequest, QueryGetInactiveLimitOrderTrancheRequest, QueryGetLimitOrderTrancheRequest, QueryGetLimitOrderTrancheUserRequest, QueryGetPoolMetadataRequest, QueryGetPoolReservesRequest, QueryParamsRequest, QueryPoolByIdRequest, QueryPoolRequest};
use crate::msg::DexMsg;

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
    Send { to: String, amount: u128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: DexMsg,
) -> StdResult<Response<NeutronMsg>> {
    execute_dex(deps, env, info, msg)
}

fn execute_dex(
    _deps: DepsMut,
    env: Env,
    _: MessageInfo,
    msg: DexMsg,
) -> StdResult<Response<NeutronMsg>> {
    let resp_msg: CosmosMsg<NeutronMsg>;

    match msg {
        DexMsg::Deposit { receiver, token_a, token_b, amounts_a, amounts_b, tick_indexes_a_to_b, fees, options } => {
            let req = dex::MsgDeposit{
                creator: env.contract.address.to_string(),
                receiver,
                token_a,
                token_b,
                amounts_a,
                amounts_b,
                tick_indexes_a_to_b,
                fees,
                options: vec![dex::DepositOptions{ disable_autoswap: true }],
            };
            resp_msg = CosmosMsg::Stargate{ type_url: "/neutron.dex.MsgDeposit".to_string(), value: Binary::from(req.encode_to_vec())};
        }
        DexMsg::Withdrawal { receiver, token_a, token_b, shares_to_remove, tick_indexes_a_to_b, fees } => {
            let req = dex::MsgWithdrawal{
                creator: env.contract.address.to_string(),
                receiver,
                token_a,
                token_b,
                shares_to_remove,
                tick_indexes_a_to_b,
                fees,
            };
            resp_msg = CosmosMsg::Stargate{ type_url: "/neutron.dex.MsgWithdrawal".to_string(), value: Binary::from(req.encode_to_vec())};
        }
        DexMsg::PlaceLimitOrder { receiver, token_in, token_out, tick_index_in_to_out, amount_in, order_type, max_amount_out } => {
            let req = dex::MsgPlaceLimitOrder{
                creator: env.contract.address.to_string(),
                receiver,
                token_in,
                token_out,
                tick_index_in_to_out,
                amount_in,
                order_type,
                expiration_time: None,
                max_amount_out,
            };
            resp_msg = CosmosMsg::Stargate{ type_url: "/neutron.dex.MsgPlaceLimitOrder".to_string(), value: Binary::from(req.encode_to_vec())};
        }
        DexMsg::WithdrawFilledLimitOrder { tranche_key } => {
            let req = dex::MsgWithdrawFilledLimitOrder{ creator: env.contract.address.to_string(), tranche_key };
            resp_msg = CosmosMsg::Stargate{ type_url: "/neutron.dex.MsgWithdrawFilledLimitOrder".to_string(), value: Binary::from(req.encode_to_vec())};
        }
        DexMsg::CancelLimitOrder { tranche_key } => {
            let req = dex::MsgCancelLimitOrder{ creator: env.contract.address.to_string(), tranche_key };
            resp_msg = CosmosMsg::Stargate{ type_url: "/neutron.dex.MsgCancelLimitOrder".to_string(), value: Binary::from(req.encode_to_vec())};
        }
        DexMsg::MultiHopSwap { receiver, routes, amount_in, exit_limit_price, pick_best_route } => {
            let req = dex::MsgMultiHopSwap{
                creator: env.contract.address.to_string(),
                receiver,
                routes: routes.iter().map(|r| r.into()).collect(),
                amount_in,
                exit_limit_price,
                pick_best_route,
            };
            resp_msg = CosmosMsg::Stargate{ type_url: "/neutron.dex.MsgMultiHopSwap".to_string(), value: Binary::from(req.encode_to_vec())};
        }
    };

    Ok(Response::default().add_message(resp_msg))
}

#[entry_point]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: DexQuery) -> StdResult<Binary> {
    query_dex(deps, env, msg)
}

fn query_dex(deps: Deps<NeutronQuery>, _env: Env, msg: DexQuery) -> StdResult<Binary> {
    match msg {
        DexQuery::Params {} => {
            let req = QueryParamsRequest{};
            let resp = make_stargate_query(deps, "/neutron.dex.Query/Params".to_string(), req.encode_to_vec())?;
            return to_binary(&resp);
        }
        DexQuery::LimitOrderTrancheUser { address, tranche_key } => {
            let req = QueryGetLimitOrderTrancheUserRequest{ address, tranche_key };
            let resp = make_stargate_query(deps, "/neutron.dex.Query/LimitOrderTrancheUser".to_string(), req.encode_to_vec())?;
            return to_binary(&resp);
        }
        DexQuery::LimitOrderTrancheUserAll {} => {
            let req = QueryAllLimitOrderTrancheUserRequest{ pagination: None };
            let resp = make_stargate_query(deps, "/neutron.dex.Query/LimitOrderTrancheUser".to_string(), req.encode_to_vec())?;
            return to_binary(&resp);
        }
        DexQuery::LimitOrderTranche { pair_id, tick_index, token_in, tranche_key } => {
            let req = QueryGetLimitOrderTrancheRequest{
                pair_id,
                tick_index,
                token_in,
                tranche_key,
            };
            let resp = make_stargate_query(deps, "/neutron.dex.Query/LimitOrderTranche".to_string(), req.encode_to_vec())?;
            return to_binary(&resp);
        }
        DexQuery::LimitOrderTrancheAll { pair_id, token_in} => {
            let req = QueryAllLimitOrderTrancheRequest{
                pair_id,
                token_in,
                pagination: None,
            };
            let resp = make_stargate_query(deps, "/neutron.dex.Query/LimitOrderTrancheAll".to_string(), req.encode_to_vec())?;
            return to_binary(&resp);
        }
        DexQuery::UserDepositAll { address} => {
            let req = QueryAllUserDepositsRequest{ address, pagination: None };
            let resp = make_stargate_query(deps, "/neutron.dex.Query/UserDepositsAll".to_string(), req.encode_to_vec())?;
            return to_binary(&resp);
        }
        DexQuery::TickLiquidityAll { pair_id, token_in} => {
            let req = QueryAllTickLiquidityRequest{
                pair_id,
                token_in,
                pagination: None,
            };
            let resp = make_stargate_query(deps, "/neutron.dex.Query/TickLiquidityAll".to_string(), req.encode_to_vec())?;
            return to_binary(&resp);
        }
        DexQuery::InactiveLimitOrderTranche { pair_id, tick_index, token_in, tranche_key } => {
            let req = QueryGetInactiveLimitOrderTrancheRequest{
                pair_id,
                token_in,
                tick_index,
                tranche_key,
            };
            let resp = make_stargate_query(deps, "/neutron.dex.Query/InactiveLimitOrderTranche".to_string(), req.encode_to_vec())?;
            return to_binary(&resp);
        }
        DexQuery::InactiveLimitOrderTrancheAll {} => {
            let req = QueryAllInactiveLimitOrderTrancheRequest{ pagination: None };
            let resp = make_stargate_query(deps, "/neutron.dex.Query/InactiveLimitOrderTrancheAll".to_string(), req.encode_to_vec())?;
            return to_binary(&resp);
        }
        DexQuery::PoolReservesAll { pair_id, token_in} => {
            let req = QueryAllPoolReservesRequest{
                pair_id,
                token_in,
                pagination: None,
            };
            let resp = make_stargate_query(deps, "/neutron.dex.Query/PoolReservesAll".to_string(), req.encode_to_vec())?;
            return to_binary(&resp);
        }
        DexQuery::PoolReserves { pair_id, token_in, tick_index, fee } => {
            let req = QueryGetPoolReservesRequest{
                pair_id,
                token_in,
                tick_index,
                fee,
            };
            let resp = make_stargate_query(deps, "/neutron.dex.Query/PoolReserves".to_string(), req.encode_to_vec())?;
            return to_binary(&resp);
        }
        DexQuery::EstimateMultiHopSwap { creator, receiver, routes, amount_in, exit_limit_price, pick_best_route } => {
            let req = QueryEstimateMultiHopSwapRequest{
                creator,
                receiver,
                routes: routes.iter().map(|r| r.into()).collect(),
                amount_in,
                exit_limit_price,
                pick_best_route,
            };
            let resp = make_stargate_query(deps, "/neutron.dex.Query/EstimateMultiHopSwap".to_string(), req.encode_to_vec())?;
            return to_binary(&resp);
        }
        DexQuery::EstimatePlaceLimitOrder { creator, receiver, token_in, token_out, tick_index_in_to_out, expiration_time, max_amount_out } => {
            let req = QueryEstimatePlaceLimitOrderRequest{
                creator,
                receiver,
                token_in,
                token_out,
                tick_index_in_to_out: 0,
                amount_in: "".to_string(),
                order_type: 0,
                expiration_time: None,
                max_amount_out: "".to_string(),
            };
            let resp = make_stargate_query(deps, "/neutron.dex.Query/EstimateMultiHopSwap".to_string(), req.encode_to_vec())?;
            return to_binary(&resp);
        }
        DexQuery::Pool { pair_id, tick_index, fee } => {
            let req = QueryPoolRequest{
                pair_id,
                tick_index,
                fee,
            };
            let resp = make_stargate_query(deps, "/neutron.dex.Query/Pool".to_string(), req.encode_to_vec())?;
            return to_binary(&resp);
        }
        DexQuery::PoolByID { pool_id } => {
            let req = QueryPoolByIdRequest{ pool_id };
            let resp = make_stargate_query(deps, "/neutron.dex.Query/PoolByID".to_string(), req.encode_to_vec())?;
            return to_binary(&resp);
        }
        DexQuery::PoolMetadata { id } => {
            let req = QueryGetPoolMetadataRequest{ id };
            let resp = make_stargate_query(deps, "/neutron.dex.Query/PoolMetadata".to_string(), req.encode_to_vec())?;
            return to_binary(&resp);
        }
        DexQuery::PoolMetadataAll { } => {
            let req = QueryAllPoolMetadataRequest{ pagination: None };
            let resp = make_stargate_query(deps, "/neutron.dex.Query/PoolMetadataAll".to_string(), req.encode_to_vec())?;
            return to_binary(&resp);
        }
        DexQuery::LimitOrderTrancheUserAllByAddress { address } => {
            let req = QueryAllUserLimitOrdersRequest{ address, pagination: None };
            let resp = make_stargate_query(deps, "/neutron.dex.Query/LimitOrderTrancheUserAllByAddress".to_string(), req.encode_to_vec())?;
            return to_binary(&resp);
        }
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

pub fn make_stargate_query(
    deps: Deps<NeutronQuery>,
    path: String,
    encoded_query_data: Vec<u8>,
) -> StdResult<String> {
    let raw = to_vec::<QueryRequest<Empty>>(&QueryRequest::Stargate {
        path,
        data: encoded_query_data.into(),
    })
        .map_err(|serialize_err| {
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
        // response(value) is base64 encoded bytes
        SystemResult::Ok(ContractResult::Ok(value)) => {
            let str = value.to_base64();
            deps.api
                .debug(format!("WASMDEBUG: make_stargate_query: {:?}", str).as_str());
            from_utf8(value.as_slice())
                .map(|s| s.to_string())
                .map_err(|_e| StdError::generic_err("Unable to encode from utf8"))
        }
    }
}