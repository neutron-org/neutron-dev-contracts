use cosmwasm_std::{
    coin, entry_point, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult,
};
use cw2::set_contract_version;
use neutron_sdk::bindings::msg::MsgIbcTransferResponse;
use neutron_sdk::{
    bindings::msg::{IbcFee, NeutronMsg},
    sudo::msg::{RequestPacket, RequestPacketTimeoutHeight, TransferSudoMsg},
};
use neutron_sdk::sudo::msg::SudoMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{
    IBC_FEE,
};

// Default timeout for IbcTransfer is 10000000 blocks
const DEFAULT_TIMEOUT_HEIGHT: u64 = 10000000;

const CONTRACT_NAME: &str = concat!("crates.io:neutron-contracts__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}

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
    Send {
        channel: String,
        to: String,
        denom: String,
        amount: u128,
        timeout_height: Option<u64>,
        memo: Option<String>,
    },
    SetFees {
        recv_fee: u128,
        ack_fee: u128,
        timeout_fee: u128,
        denom: String,
    },
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _: MessageInfo,
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
        } => execute_send(deps, env, channel, to, denom, amount, timeout_height, memo),
        ExecuteMsg::SetFees {
            recv_fee,
            ack_fee,
            timeout_fee,
            denom,
        } => execute_set_fees(deps, recv_fee, ack_fee, timeout_fee, denom),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
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
    use cosmwasm_std::Addr;

    use super::*;

    pub fn ibc_ack(
        deps: DepsMut,
        contract: Addr,
        _success: bool,
    ) -> Result<Response, ContractError> {
        // TODO
        Ok(Response::new().add_attribute("action", "ack"))
    }

    pub(crate) fn ibc_timeout(deps: DepsMut, contract: Addr) -> Result<Response, ContractError> {
        // TODO
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

fn execute_send(
    mut deps: DepsMut,
    env: Env,
    channel: String,
    to: String,
    denom: String,
    amount: u128,
    timeout_height: Option<u64>,
    maybe_memo: Option<String>,
) -> StdResult<Response<NeutronMsg>> {
    let fee = IBC_FEE.load(deps.storage)?;
    let coin1 = coin(amount, denom.clone());
    let mut msg = NeutronMsg::IbcTransfer {
        source_port: "transfer".to_string(),
        source_channel: channel.clone(),
        sender: env.contract.address.to_string(),
        receiver: to.clone(),
        token: coin1,
        timeout_height: RequestPacketTimeoutHeight {
            revision_number: Some(2),
            revision_height: timeout_height.or(Some(DEFAULT_TIMEOUT_HEIGHT)),
        },
        timeout_timestamp: 0,
        fee: fee.clone(),
        memo: maybe_memo.unwrap_or_default(),
    };

    deps.as_ref()
        .api
        .debug(format!("WASMDEBUG: execute_send: sent msg: {msg:?}").as_str());
    Ok(Response::default().add_message(msg))
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
