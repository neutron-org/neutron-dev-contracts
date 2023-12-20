use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult,
};
use cw2::set_contract_version;
use neutron_sdk::{
    bindings::msg::NeutronMsg,
    stargate::dex::msg::{
        msg_cancel_limit_order, msg_deposit, msg_multi_hop_swap, msg_place_limit_order,
        msg_withdraw_filled_limit_order, msg_withdrawal,
    },
    stargate::dex::query::{
        get_estimate_multi_hop_swap, get_estimate_place_limit_order,
        get_inactive_limit_order_tranche, get_inactive_limit_order_tranche_all,
        get_limit_order_tranche, get_limit_order_tranche_all, get_limit_order_tranche_user,
        get_limit_order_tranche_user_all, get_limit_order_tranche_user_all_by_address, get_params,
        get_pool, get_pool_by_id, get_pool_metadata, get_pool_metadata_all, get_pool_reserves,
        get_pool_reserves_all, get_tick_liquidity_all, get_user_deposits_all,
    },
    stargate::dex::types::{
        AllInactiveLimitOrderTrancheRequest, AllLimitOrderTrancheRequest, AllPoolMetadataRequest,
        AllPoolReservesRequest, AllTickLiquidityRequest, AllUserDepositsRequest,
        AllUserLimitOrdersRequest, CancelLimitOrderRequest, DepositRequest,
        EstimateMultiHopSwapRequest, EstimatePlaceLimitOrderRequest,
        GetInactiveLimitOrderTrancheRequest, GetLimitOrderTrancheRequest, GetPoolMetadataRequest,
        GetPoolReservesRequest, LimitOrderTrancheUserAllRequest, LimitOrderTrancheUserRequest,
        LimitOrderType, MultiHopSwapRequest, ParamsRequest, PlaceLimitOrderRequest,
        PoolByIdRequest, PoolRequest, WithdrawFilledLimitOrderRequest, WithdrawalRequest,
    },
    sudo::msg::SudoMsg,
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
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response<NeutronMsg>> {
    deps.api
        .debug(format!("WASMDEBUG: execute: received msg: {:?}", msg).as_str());
    match msg {
        ExecuteMsg::Deposit {
            receiver,
            token_a,
            token_b,
            amounts_a,
            amounts_b,
            tick_indexes_a_to_b,
            fees,
            options,
        } => Ok(Response::new().add_message(msg_deposit(DepositRequest {
            sender: env.contract.address.to_string(),
            receiver,
            token_a,
            token_b,
            amounts_a,
            amounts_b,
            tick_indexes_a_to_b,
            fees,
            options,
        }))),

        ExecuteMsg::Withdrawal {
            receiver,
            token_a,
            token_b,
            shares_to_remove,
            tick_indexes_a_to_b,
            fees,
        } => Ok(
            Response::new().add_message(msg_withdrawal(WithdrawalRequest {
                sender: env.contract.address.to_string(),
                receiver,
                token_a,
                token_b,
                shares_to_remove,
                tick_indexes_a_to_b,
                fees,
            })),
        ),

        ExecuteMsg::PlaceLimitOrder {
            receiver,
            token_in,
            token_out,
            tick_index_in_to_out,
            amount_in,
            order_type,
            expiration_time,
            max_amount_out,
        } => Ok(
            Response::new().add_message(msg_place_limit_order(PlaceLimitOrderRequest {
                sender: env.contract.address.to_string(),
                receiver,
                token_in,
                token_out,
                tick_index_in_to_out,
                amount_in,
                order_type: LimitOrderType::try_from(order_type).map_err(StdError::generic_err)?,
                expiration_time,
                max_amount_out,
            })),
        ),

        ExecuteMsg::WithdrawFilledLimitOrder { tranche_key } => Ok(Response::new().add_message(
            msg_withdraw_filled_limit_order(WithdrawFilledLimitOrderRequest {
                sender: env.contract.address.to_string(),
                tranche_key,
            }),
        )),

        ExecuteMsg::CancelLimitOrder { tranche_key } => Ok(Response::new().add_message(
            msg_cancel_limit_order(CancelLimitOrderRequest {
                sender: env.contract.address.to_string(),
                tranche_key,
            }),
        )),

        ExecuteMsg::MultiHopSwap {
            receiver,
            routes,
            amount_in,
            exit_limit_price,
            pick_best_route,
        } => Ok(
            Response::new().add_message(msg_multi_hop_swap(MultiHopSwapRequest {
                sender: env.contract.address.to_string(),
                receiver,
                routes,
                amount_in,
                exit_limit_price,
                pick_best_route,
            })),
        ),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    deps.api
        .debug(format!("WASMDEBUG: query: received msg: {:?}", msg).as_str());
    match msg {
        QueryMsg::Params {} => Ok(to_json_binary(&get_params(deps, ParamsRequest {})?)?),

        QueryMsg::GetLimitOrderTrancheUser {
            address,
            tranche_key,
        } => Ok(to_json_binary(&get_limit_order_tranche_user(
            deps,
            LimitOrderTrancheUserRequest {
                address,
                tranche_key,
            },
        )?)?),

        QueryMsg::AllLimitOrderTrancheUser { pagination } => {
            Ok(to_json_binary(&get_limit_order_tranche_user_all(
                deps,
                LimitOrderTrancheUserAllRequest { pagination },
            )?)?)
        }

        QueryMsg::AllLimitOrderTrancheUserByAddress {
            address,
            pagination,
        } => Ok(to_json_binary(
            &get_limit_order_tranche_user_all_by_address(
                deps,
                AllUserLimitOrdersRequest {
                    address,
                    pagination,
                },
            )?,
        )?),

        QueryMsg::GetLimitOrderTranche {
            pair_id,
            tick_index,
            token_in,
            tranche_key,
        } => Ok(to_json_binary(&get_limit_order_tranche(
            deps,
            GetLimitOrderTrancheRequest {
                pair_id,
                tick_index,
                token_in,
                tranche_key,
            },
        )?)?),

        QueryMsg::AllLimitOrderTranche {
            pair_id,
            token_in,
            pagination,
        } => Ok(to_json_binary(&get_limit_order_tranche_all(
            deps,
            AllLimitOrderTrancheRequest {
                pair_id,
                token_in,
                pagination,
            },
        )?)?),

        QueryMsg::AllUserDeposits {
            address,
            pagination,
        } => Ok(to_json_binary(&get_user_deposits_all(
            deps,
            AllUserDepositsRequest {
                address,
                pagination,
            },
        )?)?),

        QueryMsg::AllTickLiquidity {
            pair_id,
            token_in,
            pagination,
        } => Ok(to_json_binary(&get_tick_liquidity_all(
            deps,
            AllTickLiquidityRequest {
                pair_id,
                token_in,
                pagination,
            },
        )?)?),

        QueryMsg::GetInactiveLimitOrderTranche {
            pair_id,
            token_in,
            tick_index,
            tranche_key,
        } => Ok(to_json_binary(&get_inactive_limit_order_tranche(
            deps,
            GetInactiveLimitOrderTrancheRequest {
                pair_id,
                token_in,
                tick_index,
                tranche_key,
            },
        )?)?),

        QueryMsg::AllInactiveLimitOrderTranche { pagination } => {
            Ok(to_json_binary(&get_inactive_limit_order_tranche_all(
                deps,
                AllInactiveLimitOrderTrancheRequest { pagination },
            )?)?)
        }

        QueryMsg::AllPoolReserves {
            pair_id,
            token_in,
            pagination,
        } => Ok(to_json_binary(&get_pool_reserves_all(
            deps,
            AllPoolReservesRequest {
                pair_id,
                token_in,
                pagination,
            },
        )?)?),

        QueryMsg::GetPoolReserves {
            pair_id,
            token_in,
            tick_index,
            fee,
        } => Ok(to_json_binary(&get_pool_reserves(
            deps,
            GetPoolReservesRequest {
                pair_id,
                token_in,
                tick_index,
                fee,
            },
        )?)?),

        QueryMsg::EstimateMultiHopSwap {
            creator,
            receiver,
            routes,
            amount_in,
            exit_limit_price,
            pick_best_route,
        } => Ok(to_json_binary(&get_estimate_multi_hop_swap(
            deps,
            EstimateMultiHopSwapRequest {
                creator,
                receiver,
                routes,
                amount_in,
                exit_limit_price,
                pick_best_route,
            },
        )?)?),

        QueryMsg::EstimatePlaceLimitOrder {
            creator,
            receiver,
            token_in,
            token_out,
            tick_index_in_to_out,
            amount_in,
            order_type,
            expiration_time,
            max_amount_out,
        } => Ok(to_json_binary(&get_estimate_place_limit_order(
            deps,
            EstimatePlaceLimitOrderRequest {
                creator,
                receiver,
                token_in,
                token_out,
                tick_index_in_to_out,
                amount_in,
                order_type: LimitOrderType::try_from(order_type).map_err(StdError::generic_err)?,
                expiration_time,
                max_amount_out,
            },
        )?)?),

        QueryMsg::Pool {
            pair_id,
            tick_index,
            fee,
        } => Ok(to_json_binary(&get_pool(
            deps,
            PoolRequest {
                pair_id,
                tick_index,
                fee,
            },
        )?)?),

        QueryMsg::PoolById { pool_id } => Ok(to_json_binary(&get_pool_by_id(
            deps,
            PoolByIdRequest { pool_id },
        )?)?),

        QueryMsg::GetPoolMetadata { id } => Ok(to_json_binary(&get_pool_metadata(
            deps,
            GetPoolMetadataRequest { id },
        )?)?),

        QueryMsg::AllPoolMetadata { pagination } => Ok(to_json_binary(&get_pool_metadata_all(
            deps,
            AllPoolMetadataRequest { pagination },
        )?)?),
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
