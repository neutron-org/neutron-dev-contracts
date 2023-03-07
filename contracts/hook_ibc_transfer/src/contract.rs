use crate::msg::{
    ExecuteMsg, IBCLifecycleComplete, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg,
    TestArgResponse,
};
use cosmwasm_std::{
    coin, entry_point, to_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult,
};
use cw2::set_contract_version;
use neutron_sdk::{
    bindings::msg::{IbcFee, NeutronMsg},
    sudo::msg::RequestPacketTimeoutHeight,
};

use crate::state::{IBC_FEE, TEST_ARGS};

// Default timeout for IbcTransfer is 10000000 blocks
const DEFAULT_TIMEOUT_HEIGHT: u64 = 10000000;

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
    msg: ExecuteMsg,
) -> StdResult<Response<NeutronMsg>> {
    deps.api
        .debug(format!("WASMDEBUG: execute: received msg: {:?}", msg).as_str());
    match msg {
        ExecuteMsg::Send {
            channel,
            to,
            denom,
            amount,
            timeout_height,
            memo,
        } => execute_send(
            deps,
            env,
            channel,
            TransferArgs {
                to,
                denom,
                amount,
                timeout_height,
                memo,
            },
        ),
        ExecuteMsg::SetFees {
            recv_fee,
            ack_fee,
            timeout_fee,
            denom,
        } => execute_set_fees(deps, recv_fee, ack_fee, timeout_fee, denom),
        ExecuteMsg::TestMsg { return_err, arg } => execute_test_arg(deps, info, return_err, arg),
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::TestMsg { arg } => to_binary(&query_test_msg(deps, env, arg)?),
    }
}

fn query_test_msg(deps: Deps, _env: Env, arg: String) -> StdResult<TestArgResponse> {
    let sender = TEST_ARGS.may_load(deps.storage, &arg)?.unwrap_or_default();
    Ok(TestArgResponse { sender })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> StdResult<Response> {
    match msg {
        SudoMsg::IBCLifecycleComplete(IBCLifecycleComplete::IBCAck {
            channel: _,
            sequence: _,
            ack: _,
            success,
        }) => sudo::ibc_ack(deps, env.contract.address, success),
        SudoMsg::IBCLifecycleComplete(IBCLifecycleComplete::IBCTimeout {
            channel: _,
            sequence: _,
        }) => sudo::ibc_timeout(deps, env.contract.address),
    }
}

pub mod sudo {
    use crate::state::{IbcTestAck, IBC_TEST_ACKS};
    use cosmwasm_std::Addr;

    use super::*;

    pub fn ibc_ack(deps: DepsMut, _contract: Addr, success: bool) -> StdResult<Response> {
        IBC_TEST_ACKS.save(deps.storage, &IbcTestAck::Ack(success))?;
        Ok(Response::new().add_attribute("action", "ack"))
    }

    pub(crate) fn ibc_timeout(deps: DepsMut, _contract: Addr) -> StdResult<Response> {
        IBC_TEST_ACKS.save(deps.storage, &IbcTestAck::Timeout)?;
        Ok(Response::new().add_attribute("action", "timeout"))
    }
}

fn get_fee_item(denom: String, amount: u128) -> Vec<Coin> {
    if amount == 0 {
        vec![]
    } else {
        vec![coin(amount, denom)]
    }
}

fn execute_set_fees(
    deps: DepsMut,
    recv_fee: u128,
    ack_fee: u128,
    timeout_fee: u128,
    denom: String,
) -> StdResult<Response<NeutronMsg>> {
    let fee = IbcFee {
        recv_fee: get_fee_item(denom.clone(), recv_fee),
        ack_fee: get_fee_item(denom.clone(), ack_fee),
        timeout_fee: get_fee_item(denom, timeout_fee),
    };

    IBC_FEE.save(deps.storage, &fee)?;

    Ok(Response::default())
}

fn execute_test_arg(
    deps: DepsMut,
    info: MessageInfo,
    return_err: bool,
    arg: String,
) -> StdResult<Response<NeutronMsg>> {
    if return_err {
        return Err(StdError::generic_err("return error"));
    }

    TEST_ARGS.save(deps.storage, &arg, &info.sender.to_string())?;

    Ok(Response::new().add_attribute("arg", arg))
}

fn execute_send(
    deps: DepsMut,
    env: Env,
    channel: String,
    args: TransferArgs,
) -> StdResult<Response<NeutronMsg>> {
    let msg = NeutronMsg::IbcTransfer {
        source_port: "transfer".to_string(),
        source_channel: channel,
        sender: env.contract.address.to_string(),
        receiver: args.to,
        token: coin(args.amount, args.denom),
        timeout_height: RequestPacketTimeoutHeight {
            revision_number: Some(2),
            revision_height: args.timeout_height.or(Some(DEFAULT_TIMEOUT_HEIGHT)),
        },
        timeout_timestamp: 0,
        fee: IBC_FEE.load(deps.storage)?,
        memo: args.memo.unwrap_or_default(),
    };

    deps.as_ref()
        .api
        .debug(format!("WASMDEBUG: execute_send: sent msg: {msg:?}").as_str());
    Ok(Response::default().add_message(msg))
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}

struct TransferArgs {
    pub to: String,
    pub denom: String,
    pub amount: u128,
    pub timeout_height: Option<u64>,
    pub memo: Option<String>,
}
