use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use neutron_sdk::sudo::msg::SudoMsg;
use neutron_std::types::neutron::dex::{
    DexQuerier, MsgCancelLimitOrder, MsgDeposit, MsgMultiHopSwap, MsgPlaceLimitOrder,
    MsgWithdrawFilledLimitOrder, MsgWithdrawal,
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
) -> StdResult<Response> {
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
        } => Ok(Response::new().add_message(MsgDeposit {
            creator: env.contract.address.to_string(),
            receiver,
            token_a,
            token_b,
            amounts_a,
            amounts_b,
            tick_indexes_a_to_b,
            fees,
            options,
        })),

        ExecuteMsg::Withdrawal {
            receiver,
            token_a,
            token_b,
            shares_to_remove,
            tick_indexes_a_to_b,
            fees,
        } => Ok(Response::new().add_message(MsgWithdrawal {
            creator: env.contract.address.to_string(),
            receiver,
            token_a,
            token_b,
            shares_to_remove,
            tick_indexes_a_to_b,
            fees,
        })),
        #[allow(deprecated)]
        ExecuteMsg::PlaceLimitOrder {
            receiver,
            token_in,
            token_out,
            tick_index_in_to_out,
            limit_sell_price,
            amount_in,
            order_type,
            expiration_time,
            max_amount_out,
        } => Ok(Response::new().add_message(MsgPlaceLimitOrder {
            creator: env.contract.address.to_string(),
            receiver,
            token_in,
            token_out,
            tick_index_in_to_out,
            limit_sell_price,
            amount_in,
            order_type,
            expiration_time,
            max_amount_out,
            min_average_sell_price: "".to_string(), // TODO
        })),
        ExecuteMsg::WithdrawFilledLimitOrder { tranche_key } => {
            Ok(Response::new().add_message(MsgWithdrawFilledLimitOrder {
                creator: env.contract.address.to_string(),
                tranche_key,
            }))
        }

        ExecuteMsg::CancelLimitOrder { tranche_key } => {
            Ok(Response::new().add_message(MsgCancelLimitOrder {
                creator: env.contract.address.to_string(),
                tranche_key,
            }))
        }

        ExecuteMsg::MultiHopSwap {
            receiver,
            routes,
            amount_in,
            exit_limit_price,
            pick_best_route,
        } => Ok(Response::new().add_message(MsgMultiHopSwap {
            creator: env.contract.address.to_string(),
            receiver,
            routes,
            amount_in,
            exit_limit_price,
            pick_best_route,
        })),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    deps.api
        .debug(format!("WASMDEBUG: query: received msg: {:?}", msg).as_str());

    let dex_querier = DexQuerier::new(&deps.querier);

    match msg {
        QueryMsg::Params {} => Ok(to_json_binary(&dex_querier.params()?)?),

        QueryMsg::GetLimitOrderTrancheUser {
            address,
            tranche_key,
            calc_withdrawable_shares,
        } => Ok(to_json_binary(&dex_querier.limit_order_tranche_user(
            address,
            tranche_key,
            calc_withdrawable_shares,
        )?)?),

        QueryMsg::AllLimitOrderTrancheUser { pagination } => Ok(to_json_binary(
            &dex_querier.limit_order_tranche_user_all(pagination)?,
        )?),

        QueryMsg::AllLimitOrderTrancheUserByAddress {
            address,
            pagination,
        } => Ok(to_json_binary(
            &dex_querier.limit_order_tranche_user_all_by_address(address, pagination)?,
        )?),

        QueryMsg::GetLimitOrderTranche {
            pair_id,
            tick_index,
            token_in,
            tranche_key,
        } => Ok(to_json_binary(&dex_querier.limit_order_tranche(
            pair_id,
            tick_index,
            token_in,
            tranche_key,
        )?)?),

        QueryMsg::AllLimitOrderTranche {
            pair_id,
            token_in,
            pagination,
        } => Ok(to_json_binary(
            &dex_querier.limit_order_tranche_all(pair_id, token_in, pagination)?,
        )?),

        QueryMsg::AllUserDeposits {
            address,
            include_pool_data,
            pagination,
        } => Ok(to_json_binary(&dex_querier.user_deposits_all(
            address,
            pagination,
            include_pool_data,
        )?)?),

        QueryMsg::AllTickLiquidity {
            pair_id,
            token_in,
            pagination,
        } => Ok(to_json_binary(
            &dex_querier.tick_liquidity_all(pair_id, token_in, pagination)?,
        )?),

        QueryMsg::GetInactiveLimitOrderTranche {
            pair_id,
            token_in,
            tick_index,
            tranche_key,
        } => Ok(to_json_binary(&dex_querier.inactive_limit_order_tranche(
            pair_id,
            token_in,
            tick_index,
            tranche_key,
        )?)?),

        QueryMsg::AllInactiveLimitOrderTranche { pagination } => Ok(to_json_binary(
            &dex_querier.inactive_limit_order_tranche_all(pagination)?,
        )?),

        QueryMsg::AllPoolReserves {
            pair_id,
            token_in,
            pagination,
        } => Ok(to_json_binary(
            &dex_querier.pool_reserves_all(pair_id, token_in, pagination)?,
        )?),

        QueryMsg::GetPoolReserves {
            pair_id,
            token_in,
            tick_index,
            fee,
        } => Ok(to_json_binary(
            &dex_querier.pool_reserves(pair_id, token_in, tick_index, fee)?,
        )?),

        #[allow(deprecated)] // Allow deprecated call until its remove from neutron-core
        QueryMsg::EstimateMultiHopSwap {
            creator,
            receiver,
            routes,
            amount_in,
            exit_limit_price,
            pick_best_route,
        } => Ok(to_json_binary(&dex_querier.estimate_multi_hop_swap(
            creator,
            receiver,
            routes,
            amount_in,
            exit_limit_price,
            pick_best_route,
        )?)?),

        #[allow(deprecated)] // Allow deprecated call until its remove from neutron-core
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
        } => Ok(to_json_binary(&dex_querier.estimate_place_limit_order(
            creator,
            receiver,
            token_in,
            token_out,
            tick_index_in_to_out,
            amount_in,
            order_type,
            expiration_time,
            max_amount_out,
        )?)?),

        QueryMsg::Pool {
            pair_id,
            tick_index,
            fee,
        } => Ok(to_json_binary(
            &dex_querier.pool(pair_id, tick_index, fee)?,
        )?),

        QueryMsg::PoolById { pool_id } => Ok(to_json_binary(&dex_querier.pool_by_id(pool_id)?)?),

        QueryMsg::GetPoolMetadata { id } => Ok(to_json_binary(&dex_querier.pool_metadata(id)?)?),

        QueryMsg::AllPoolMetadata { pagination } => {
            Ok(to_json_binary(&dex_querier.pool_metadata_all(pagination)?)?)
        }

        QueryMsg::SimulateDeposit { msg } => {
            Ok(to_json_binary(&dex_querier.simulate_deposit(Some(msg))?)?)
        }

        QueryMsg::SimulateWithdrawal { msg } => Ok(to_json_binary(
            &dex_querier.simulate_withdrawal(Some(msg))?,
        )?),

        QueryMsg::SimulatePlaceLimitOrder { msg } => Ok(to_json_binary(
            &dex_querier.simulate_place_limit_order(Some(msg))?,
        )?),

        QueryMsg::SimulateWithdrawFilledLimitOrder { msg } => Ok(to_json_binary(
            &dex_querier.simulate_withdraw_filled_limit_order(Some(msg))?,
        )?),

        QueryMsg::SimulateCancelLimitOrder { msg } => Ok(to_json_binary(
            &dex_querier.simulate_cancel_limit_order(Some(msg))?,
        )?),

        QueryMsg::SimulateMultiHopSwap { msg } => Ok(to_json_binary(
            &dex_querier.simulate_multi_hop_swap(Some(msg))?,
        )?),
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
