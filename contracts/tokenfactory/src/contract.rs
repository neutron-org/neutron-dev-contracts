use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cosmwasm_std::{
    coins, entry_point, to_json_binary, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult,
};
use neutron_sdk::query::token_factory::query_before_send_hook;
use neutron_sdk::{
    bindings::{msg::NeutronMsg, query::NeutronQuery},
    query::token_factory::{query_denom_admin, query_full_denom},
    NeutronResult,
};

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    _deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response<NeutronMsg>> {
    let msg: CosmosMsg<NeutronMsg> = match msg {
        ExecuteMsg::CreateDenom { subdenom } => NeutronMsg::submit_create_denom(subdenom).into(),
        ExecuteMsg::ChangeAdmin {
            denom,
            new_admin_address,
        } => NeutronMsg::submit_change_admin(denom, new_admin_address).into(),
        ExecuteMsg::MintTokens {
            denom,
            amount,
            mint_to_address,
        } => NeutronMsg::submit_mint_tokens(
            denom,
            amount,
            mint_to_address.unwrap_or(env.contract.address.into()),
        )
        .into(),
        ExecuteMsg::BurnTokens {
            denom,
            amount,
            burn_from_address,
        } => NeutronMsg::submit_burn_tokens(denom, amount, burn_from_address).into(),
        ExecuteMsg::SetBeforeSendHook {
            denom,
            contract_addr,
        } => NeutronMsg::submit_set_before_send_hook(denom, contract_addr).into(),
        ExecuteMsg::SendTokens {
            recipient,
            denom,
            amount,
        } => BankMsg::Send {
            to_address: recipient,
            amount: coins(amount.u128(), denom),
        }
        .into(),
        ExecuteMsg::ForceTransfer {
            denom,
            amount,
            from,
            to,
        } => NeutronMsg::submit_force_transfer(denom, amount, from, to).into(),
        ExecuteMsg::SetDenomMetadata {
            description,
            denom_units,
            base,
            display,
            name,
            symbol,
            uri,
            uri_hash,
        } => NeutronMsg::submit_set_denom_metadata(
            description,
            denom_units,
            base,
            display,
            name,
            symbol,
            uri,
            uri_hash,
        )
        .into(),
    };
    Ok(Response::new().add_message(msg))
}

#[entry_point]
pub fn query(deps: Deps<NeutronQuery>, _env: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    Ok(match msg {
        QueryMsg::FullDenom {
            creator_addr,
            subdenom,
        } => to_json_binary(&query_full_denom(deps, creator_addr, subdenom)?)?,
        QueryMsg::DenomAdmin { subdenom } => to_json_binary(&query_denom_admin(deps, subdenom)?)?,
        QueryMsg::BeforeSendHook { denom } => {
            to_json_binary(&query_before_send_hook(deps, denom)?)?
        }
    })
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::new())
}
